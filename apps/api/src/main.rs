#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use x402edit_api_contract::{AckRequest, JobCreateRequest, Problem, QuoteRequest};
use x402edit_application::{AppError, AppService, ServiceConfig};
use x402edit_domain::{JobId, RequestId};

type Shared = Arc<AppService>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    let state = Arc::new(AppService::new(ServiceConfig::default()));
    let app = router(state);
    let address: SocketAddr = "127.0.0.1:8080".parse().expect("valid address");
    tracing::info!(%address, "x402edit API listening (Base Sepolia; payment edge unconfigured)");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind API");
    axum::serve(listener, app).await.expect("serve API");
}

fn router(state: Shared) -> Router {
    Router::new()
        .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/quotes", post(create_quote))
        .route("/v1/jobs", post(create_job))
        .route("/v1/jobs/{id}", get(get_job))
        .route("/v1/jobs/{id}:commit", post(commit_job))
        .route("/v1/jobs/{id}:cancel", post(cancel_job))
        .route("/v1/jobs/{id}:ack", post(ack_job))
        .layer(RequestBodyLimitLayer::new(70 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn capabilities() -> impl IntoResponse {
    Json(json!({
        "schema_version": "1",
        "operations": ["generate", "edit", "design"],
        "input_formats": ["png", "jpeg", "webp"],
        "payment": {"network": "eip155:84532", "scheme": "upto", "mainnet_enabled": false},
        "privacy": {"default": "strict_ephemeral", "strict_providers": []},
        "live_provider_execution": false
    }))
}

async fn create_quote(
    State(service): State<Shared>,
    Json(body): Json<QuoteRequest>,
) -> Result<impl IntoResponse, ApiProblem> {
    Ok((StatusCode::CREATED, Json(service.quote(body)?)))
}

async fn create_job(
    State(service): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<JobCreateRequest>,
) -> Result<impl IntoResponse, ApiProblem> {
    require_idempotency(&headers)?;
    headers.get("PAYMENT-SIGNATURE").ok_or_else(|| {
        ApiProblem::new(
            StatusCode::PAYMENT_REQUIRED,
            "payment_required",
            "PAYMENT-SIGNATURE is required",
        )
    })?;
    let mock_enabled = std::env::var("X402EDIT_ENV").as_deref() == Ok("development")
        && std::env::var("X402EDIT_PAYMENT_MODE").as_deref() == Ok("mock");
    if !mock_enabled {
        return Err(ApiProblem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_edge_unavailable",
            "Payment verification is fail-closed until the internal x402 edge is configured",
        ));
    }
    Ok((StatusCode::CREATED, Json(service.create_job(body)?)))
}

async fn get_job(
    State(service): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiProblem> {
    let (id, capability) = auth(id, &headers)?;
    Ok(Json(service.status(&id, &capability)?))
}

async fn commit_job(
    State(service): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiProblem> {
    require_idempotency(&headers)?;
    let (id, capability) = auth(id, &headers)?;
    Ok(Json(service.commit(&id, &capability)?))
}

async fn cancel_job(
    State(service): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiProblem> {
    require_idempotency(&headers)?;
    let (id, capability) = auth(id, &headers)?;
    Ok(Json(service.cancel(&id, &capability)?))
}

async fn ack_job(
    State(service): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<AckRequest>,
) -> Result<impl IntoResponse, ApiProblem> {
    require_idempotency(&headers)?;
    let (id, capability) = auth(id, &headers)?;
    Ok(Json(service.acknowledge(
        &id,
        &capability,
        &body.ciphertext_sha256,
    )?))
}

fn require_idempotency(headers: &HeaderMap) -> Result<(), ApiProblem> {
    headers
        .get("Idempotency-Key")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            ApiProblem::new(
                StatusCode::BAD_REQUEST,
                "missing_idempotency_key",
                "Every mutation requires Idempotency-Key",
            )
        })?;
    Ok(())
}

fn auth(raw_id: String, headers: &HeaderMap) -> Result<(JobId, String), ApiProblem> {
    let id = JobId::parse(raw_id)
        .map_err(|e| ApiProblem::new(StatusCode::BAD_REQUEST, "invalid_job_id", &e.to_string()))?;
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            ApiProblem::new(
                StatusCode::UNAUTHORIZED,
                "invalid_capability",
                "A job bearer capability is required",
            )
        })?;
    Ok((id, value.to_owned()))
}

struct ApiProblem(Problem);

impl ApiProblem {
    fn new(status: StatusCode, code: &str, detail: &str) -> Self {
        Self(Problem {
            problem_type: format!("https://x402edit.dev/problems/{code}"),
            title: code.replace('_', " "),
            status: status.as_u16(),
            code: code.into(),
            detail: detail.into(),
            instance: RequestId::new().to_string(),
            retryable: false,
            metadata: None,
        })
    }
}

impl From<AppError> for ApiProblem {
    fn from(error: AppError) -> Self {
        let (status, code) = match &error {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "invalid_capability"),
            AppError::JobNotFound | AppError::QuoteNotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::QuoteExpired => (StatusCode::GONE, "quote_expired"),
            AppError::NotCancellable => (StatusCode::CONFLICT, "not_cancellable"),
            AppError::CiphertextHashMismatch => (StatusCode::CONFLICT, "ciphertext_hash_mismatch"),
            AppError::ResultNotReady => (StatusCode::CONFLICT, "result_not_ready"),
            AppError::Validation(_) | AppError::QuoteMismatch | AppError::State(_) => {
                (StatusCode::BAD_REQUEST, "invalid_request")
            }
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        Self::new(status, code, &error.to_string())
    }
}

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(self.0)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            "application/problem+json".parse().unwrap(),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
        response
    }
}
