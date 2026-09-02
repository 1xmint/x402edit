#![forbid(unsafe_code)]

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_workflow::VisualIntent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    OpenAi,
    Google,
    Bfl,
    Adobe,
    Xai,
    Ideogram,
    Recraft,
    Alibaba,
    Stability,
}

impl ProviderId {
    pub const ALL: [Self; 9] = [
        Self::OpenAi,
        Self::Google,
        Self::Bfl,
        Self::Adobe,
        Self::Xai,
        Self::Ideogram,
        Self::Recraft,
        Self::Alibaba,
        Self::Stability,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClass {
    StrictEligible,
    ConsentOnly,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderLifecycle {
    FixedSnapshot,
    StableAlias,
    MutableAlias,
    Preview,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderCapabilities {
    pub operations: Vec<Operation>,
    pub max_references: u8,
    pub supports_masks: bool,
    pub supports_vector: bool,
    pub supports_transparency: bool,
    pub supports_structured_prompt: bool,
    pub output_formats: Vec<OutputFormat>,
    pub resolutions: Vec<Resolution>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub model: String,
    pub lifecycle: ProviderLifecycle,
    pub privacy_class: PrivacyClass,
    pub policy_digest: String,
    pub capabilities: ProviderCapabilities,
    pub egress_hosts: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestEncoding {
    Json,
    Multipart,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledProviderRequest {
    pub method: String,
    pub url: Url,
    pub encoding: RequestEncoding,
    pub body: Value,
    pub idempotency_header: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSubmission {
    pub provider_request_id: String,
    pub status_url: Option<Url>,
    pub initial_state: ProviderState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderState {
    Submitted,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn descriptor(&self) -> &ProviderDescriptor;

    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError>;

    async fn submit(
        &self,
        _request: &CompiledProviderRequest,
        _attempt_id: &str,
    ) -> Result<ProviderSubmission, ProviderError> {
        Err(ProviderError::LiveTrafficDisabled(self.descriptor().id))
    }

    async fn poll(&self, _submission: &ProviderSubmission) -> Result<ProviderState, ProviderError> {
        Err(ProviderError::Unsupported("poll"))
    }

    async fn cancel(&self, _submission: &ProviderSubmission) -> Result<bool, ProviderError> {
        Ok(false)
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider {0:?} live traffic is disabled until credentials and policy gates pass")]
    LiveTrafficDisabled(ProviderId),
    #[error("provider does not support operation: {0}")]
    Unsupported(&'static str),
    #[error("invalid provider request: {0}")]
    InvalidRequest(String),
    #[error("provider outcome is unknown")]
    OutcomeUnknown,
    #[error("provider request failed: {0}")]
    Request(String),
}

pub fn default_capabilities(
    operations: Vec<Operation>,
    max_references: u8,
) -> ProviderCapabilities {
    ProviderCapabilities {
        operations,
        max_references,
        supports_masks: false,
        supports_vector: false,
        supports_transparency: false,
        supports_structured_prompt: false,
        output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
        resolutions: vec![Resolution::OneK, Resolution::TwoK],
    }
}

pub fn ensure_operation(
    descriptor: &ProviderDescriptor,
    intent: &VisualIntent,
) -> Result<(), ProviderError> {
    if descriptor
        .capabilities
        .operations
        .contains(&intent.operation)
    {
        Ok(())
    } else {
        Err(ProviderError::Unsupported("operation"))
    }
}

pub fn compiled_request(
    url: &str,
    encoding: RequestEncoding,
    body: Value,
    idempotency_header: Option<&str>,
) -> Result<CompiledProviderRequest, ProviderError> {
    Ok(CompiledProviderRequest {
        method: "POST".into(),
        url: Url::parse(url).map_err(|error| ProviderError::InvalidRequest(error.to_string()))?,
        encoding,
        body,
        idempotency_header: idempotency_header.map(str::to_owned),
    })
}
