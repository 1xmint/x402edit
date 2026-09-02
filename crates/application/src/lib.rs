#![forbid(unsafe_code)]

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use dashmap::DashMap;
use rand::RngExt;
use sha2::{Digest, Sha256};
use x402edit_api_contract::{
    DeletionReceipt, JobCreateRequest, JobCreateResponse, JobStatusResponse, PaymentRequirement,
    QuoteRequest, QuoteResponse,
};
use x402edit_domain::{JobId, JobPhase, JobSnapshot, JobState, Money, PaymentState, QuoteId};
use x402edit_validation::{validate_job, validate_quote};

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub network: String,
    pub asset: String,
    pub pay_to: String,
    pub unit_price_atomic: u64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            network: "eip155:84532".into(),
            asset: "USDC".into(),
            pay_to: "UNCONFIGURED".into(),
            unit_price_atomic: 100_000,
        }
    }
}

#[derive(Clone)]
struct QuoteRecord {
    request: QuoteRequest,
    response: QuoteResponse,
}

struct JobRecord {
    snapshot: JobSnapshot,
    capability_digest: [u8; 32],
    ciphertext_hash: Option<String>,
}

pub struct AppService {
    config: ServiceConfig,
    quotes: DashMap<String, QuoteRecord>,
    jobs: DashMap<String, JobRecord>,
}

impl AppService {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            quotes: DashMap::new(),
            jobs: DashMap::new(),
        }
    }

    pub fn quote(&self, request: QuoteRequest) -> Result<QuoteResponse, AppError> {
        validate_quote(&request).map_err(|e| AppError::Validation(e.to_string()))?;
        let canonical =
            serde_json::to_vec(&request).map_err(|e| AppError::Internal(e.to_string()))?;
        let hash = hex::encode(Sha256::digest(canonical));
        let id = QuoteId::new();
        let response = QuoteResponse {
            id: id.clone(),
            expires_at: Utc::now() + Duration::minutes(5),
            request_constraints_hash: hash,
            payment: PaymentRequirement {
                x402_version: 2,
                scheme: "upto".into(),
                network: self.config.network.clone(),
                asset: self.config.asset.clone(),
                pay_to: self.config.pay_to.clone(),
                max_amount: Money {
                    asset: self.config.asset.clone(),
                    amount_atomic: self.config.unit_price_atomic,
                    decimals: 6,
                },
                resource: "/v1/jobs".into(),
            },
            provider_policy_digests: Vec::new(),
        };
        self.quotes.insert(
            id.to_string(),
            QuoteRecord {
                request,
                response: response.clone(),
            },
        );
        Ok(response)
    }

    pub fn create_job(&self, request: JobCreateRequest) -> Result<JobCreateResponse, AppError> {
        validate_job(&request).map_err(|e| AppError::Validation(e.to_string()))?;
        let quote = self
            .quotes
            .get(request.quote_id.as_str())
            .ok_or(AppError::QuoteNotFound)?;
        if quote.response.expires_at <= Utc::now() {
            return Err(AppError::QuoteExpired);
        }
        if quote.request.operation != request.operation {
            return Err(AppError::QuoteMismatch);
        }
        let id = JobId::new();
        let mut capability = [0_u8; 32];
        rand::rng().fill(&mut capability);
        let token = URL_SAFE_NO_PAD.encode(capability);
        let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let now = Utc::now();
        let snapshot = JobSnapshot {
            id: id.clone(),
            quote_id: request.quote_id,
            operation: request.operation,
            state: JobState::AwaitingInputs,
            phase: JobPhase::Intake,
            payment_state: PaymentState::Verified,
            fencing_token: 0,
            created_at: now,
            updated_at: now,
            result_expires_at: None,
            warnings: Vec::new(),
        };
        self.jobs.insert(
            id.to_string(),
            JobRecord {
                snapshot,
                capability_digest: digest,
                ciphertext_hash: None,
            },
        );
        Ok(JobCreateResponse {
            id,
            state: JobState::AwaitingInputs,
            job_capability: token,
            input_deadline: now + Duration::minutes(5),
        })
    }

    pub fn status(&self, id: &JobId, capability: &str) -> Result<JobStatusResponse, AppError> {
        let job = self.authorized(id, capability)?;
        let s = &job.snapshot;
        Ok(JobStatusResponse {
            id: s.id.clone(),
            quote_id: s.quote_id.clone(),
            state: s.state,
            phase: s.phase,
            payment_state: s.payment_state,
            created_at: s.created_at,
            updated_at: s.updated_at,
            result_expires_at: s.result_expires_at,
            warnings: s.warnings.clone(),
        })
    }

    pub fn commit(&self, id: &JobId, capability: &str) -> Result<JobStatusResponse, AppError> {
        let mut job = self.authorized_mut(id, capability)?;
        job.snapshot
            .transition(JobState::Queued, JobPhase::Planning, Utc::now())
            .map_err(|e| AppError::State(e.to_string()))?;
        drop(job);
        self.status(id, capability)
    }

    pub fn cancel(&self, id: &JobId, capability: &str) -> Result<JobStatusResponse, AppError> {
        let mut job = self.authorized_mut(id, capability)?;
        if !matches!(
            job.snapshot.state,
            JobState::AwaitingInputs | JobState::NeedsInput | JobState::Queued
        ) {
            return Err(AppError::NotCancellable);
        }
        job.snapshot
            .transition(JobState::Cancelled, JobPhase::Purge, Utc::now())
            .map_err(|e| AppError::State(e.to_string()))?;
        drop(job);
        self.status(id, capability)
    }

    pub fn acknowledge(
        &self,
        id: &JobId,
        capability: &str,
        ciphertext_hash: &str,
    ) -> Result<DeletionReceipt, AppError> {
        let mut job = self.authorized_mut(id, capability)?;
        if job.snapshot.state != JobState::Ready {
            return Err(AppError::ResultNotReady);
        }
        if job.ciphertext_hash.as_deref() != Some(ciphertext_hash) {
            return Err(AppError::CiphertextHashMismatch);
        }
        job.snapshot
            .transition(JobState::Purged, JobPhase::Purge, Utc::now())
            .map_err(|e| AppError::State(e.to_string()))?;
        Ok(DeletionReceipt { job_id: id.clone(), purged_at: Utc::now(), local_objects_deleted: 1,
            local_keys_destroyed: 1, provider_policy_digests: vec![], provider_deletion_outcomes: vec![],
            attestation_scope: "Local x402edit objects and content keys only; no claim is made about provider memory.".into(),
            signature: "UNCONFIGURED".into() })
    }

    fn authorized(
        &self,
        id: &JobId,
        capability: &str,
    ) -> Result<dashmap::mapref::one::Ref<'_, String, JobRecord>, AppError> {
        let job = self.jobs.get(id.as_str()).ok_or(AppError::JobNotFound)?;
        let digest: [u8; 32] = Sha256::digest(capability.as_bytes()).into();
        if digest != job.capability_digest {
            return Err(AppError::Unauthorized);
        }
        Ok(job)
    }

    fn authorized_mut(
        &self,
        id: &JobId,
        capability: &str,
    ) -> Result<dashmap::mapref::one::RefMut<'_, String, JobRecord>, AppError> {
        let job = self
            .jobs
            .get_mut(id.as_str())
            .ok_or(AppError::JobNotFound)?;
        let digest: [u8; 32] = Sha256::digest(capability.as_bytes()).into();
        if digest != job.capability_digest {
            return Err(AppError::Unauthorized);
        }
        Ok(job)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("quote was not found")]
    QuoteNotFound,
    #[error("quote has expired")]
    QuoteExpired,
    #[error("request does not match quote")]
    QuoteMismatch,
    #[error("job was not found")]
    JobNotFound,
    #[error("invalid job capability")]
    Unauthorized,
    #[error("job is not cancellable after provider submission")]
    NotCancellable,
    #[error("result is not ready")]
    ResultNotReady,
    #[error("ciphertext hash does not match")]
    CiphertextHashMismatch,
    #[error("state transition failed: {0}")]
    State(String),
    #[error("internal error: {0}")]
    Internal(String),
}
