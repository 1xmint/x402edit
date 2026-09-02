#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use x402edit_domain::{
    JobId, JobPhase, JobState, Money, Operation, OutputFormat, PaymentState, PrivacyMode,
    QualityProfile, QuoteId, Resolution, VisualMode,
};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DeclaredInputs {
    pub reference_count: u8,
    pub mask_count: u8,
    pub max_total_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputRequest {
    pub count: u8,
    pub format: OutputFormat,
    pub aspect_ratio: String,
    pub resolution: Resolution,
    pub quality_profile: QualityProfile,
    pub artifact_bundle: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowLimits {
    pub max_provider_attempts: u8,
    pub max_repairs: u8,
    pub allow_partial: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrivacyRequest {
    pub mode: PrivacyMode,
    #[serde(default)]
    pub allowed_provider_policy_digests: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DeliveryEncryption {
    HpkeX25519Aes128gcm,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryQuoteRequest {
    pub encryption: DeliveryEncryption,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct QuoteRequest {
    pub schema_version: String,
    pub operation: Operation,
    pub mode: VisualMode,
    pub declared_inputs: DeclaredInputs,
    pub output: OutputRequest,
    pub workflow_limits: WorkflowLimits,
    pub privacy: PrivacyRequest,
    pub delivery: DeliveryQuoteRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PaymentRequirement {
    pub x402_version: u8,
    pub scheme: String,
    pub network: String,
    pub asset: String,
    pub pay_to: String,
    pub max_amount: Money,
    pub resource: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct QuoteResponse {
    pub id: QuoteId,
    pub expires_at: DateTime<Utc>,
    pub request_constraints_hash: String,
    pub payment: PaymentRequirement,
    pub provider_policy_digests: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LiteralText {
    pub id: String,
    pub text: String,
    pub role: String,
    pub must_be_exact: bool,
    pub language: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRole {
    SubjectIdentity,
    Product,
    Style,
    Composition,
    Palette,
    Source,
    Mask,
    Artifact,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceDeclaration {
    pub slot: String,
    pub role: ReferenceRole,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryRequest {
    pub recipient_public_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderPolicyAcceptance {
    pub provider_id: String,
    pub policy_digest: String,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrivacyAcceptance {
    pub mode: PrivacyMode,
    pub accepted: Vec<ProviderPolicyAcceptance>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JobCreateRequest {
    pub schema_version: String,
    pub quote_id: QuoteId,
    pub operation: Operation,
    pub prompt: String,
    #[serde(default)]
    pub literal_text: Vec<LiteralText>,
    #[serde(default)]
    pub references: Vec<ReferenceDeclaration>,
    pub edit: Option<Value>,
    pub delivery: DeliveryRequest,
    pub privacy_acceptance: Option<PrivacyAcceptance>,
    pub client_request_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JobCreateResponse {
    pub id: JobId,
    pub state: JobState,
    pub job_capability: String,
    pub input_deadline: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JobStatusResponse {
    pub id: JobId,
    pub quote_id: QuoteId,
    pub state: JobState,
    pub phase: JobPhase,
    pub payment_state: PaymentState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result_expires_at: Option<DateTime<Utc>>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AckRequest {
    pub ciphertext_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DeletionReceipt {
    pub job_id: JobId,
    pub purged_at: DateTime<Utc>,
    pub local_objects_deleted: u32,
    pub local_keys_destroyed: u32,
    pub provider_policy_digests: Vec<String>,
    pub provider_deletion_outcomes: Vec<String>,
    pub attestation_scope: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub code: String,
    pub detail: String,
    pub instance: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_reject_unknown_fields() {
        let value = serde_json::json!({
            "schema_version": "1",
            "operation": "generate",
            "mode": "auto",
            "declared_inputs": {"reference_count": 0, "mask_count": 0, "max_total_bytes": 0},
            "output": {
                "count": 1,
                "format": "png",
                "aspect_ratio": "1:1",
                "resolution": "1k",
                "quality_profile": "balanced",
                "artifact_bundle": true
            },
            "workflow_limits": {"max_provider_attempts": 1, "max_repairs": 1, "allow_partial": false},
            "privacy": {"mode": "strict_ephemeral", "allowed_provider_policy_digests": []},
            "delivery": {"encryption": "hpke-x25519-aes128gcm"},
            "unexpected": true
        });
        assert!(serde_json::from_value::<QuoteRequest>(value).is_err());
    }
}
