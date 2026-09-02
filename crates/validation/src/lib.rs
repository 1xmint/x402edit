#![forbid(unsafe_code)]

use thiserror::Error;
use x402edit_api_contract::{JobCreateRequest, QuoteRequest};
use x402edit_domain::{
    MAX_LITERAL_TEXT_BYTES, MAX_LITERAL_TEXT_ENTRIES, MAX_OUTPUT_COUNT, MAX_PROMPT_BYTES,
    MAX_PROVIDER_ATTEMPTS, MAX_REFERENCE_COUNT, MAX_REPAIRS, SCHEMA_VERSION_V1,
};

pub fn validate_quote(request: &QuoteRequest) -> Result<(), ValidationError> {
    if request.schema_version != SCHEMA_VERSION_V1 {
        return Err(ValidationError::UnsupportedSchema);
    }
    if request.declared_inputs.reference_count > MAX_REFERENCE_COUNT {
        return Err(ValidationError::TooManyReferences);
    }
    if request.output.count == 0 || request.output.count > MAX_OUTPUT_COUNT {
        return Err(ValidationError::InvalidOutputCount);
    }
    if request.workflow_limits.max_provider_attempts == 0
        || request.workflow_limits.max_provider_attempts > MAX_PROVIDER_ATTEMPTS
    {
        return Err(ValidationError::InvalidProviderAttempts);
    }
    if request.workflow_limits.max_repairs > MAX_REPAIRS {
        return Err(ValidationError::InvalidRepairs);
    }
    Ok(())
}

pub fn validate_job(request: &JobCreateRequest) -> Result<(), ValidationError> {
    if request.schema_version != SCHEMA_VERSION_V1 {
        return Err(ValidationError::UnsupportedSchema);
    }
    if request.prompt.len() > MAX_PROMPT_BYTES {
        return Err(ValidationError::PromptTooLarge);
    }
    if request.literal_text.len() > MAX_LITERAL_TEXT_ENTRIES {
        return Err(ValidationError::TooManyLiteralTextEntries);
    }
    let literal_bytes: usize = request
        .literal_text
        .iter()
        .map(|item| item.text.len())
        .sum();
    if literal_bytes > MAX_LITERAL_TEXT_BYTES {
        return Err(ValidationError::LiteralTextTooLarge);
    }
    if request.references.len() > MAX_REFERENCE_COUNT as usize {
        return Err(ValidationError::TooManyReferences);
    }
    if request.client_request_id.is_empty() || request.delivery.recipient_public_key.is_empty() {
        return Err(ValidationError::MissingRequiredValue);
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("unsupported schema version")]
    UnsupportedSchema,
    #[error("prompt exceeds 16 KiB")]
    PromptTooLarge,
    #[error("too many protected literal text entries")]
    TooManyLiteralTextEntries,
    #[error("protected literal text exceeds 8 KiB")]
    LiteralTextTooLarge,
    #[error("too many references")]
    TooManyReferences,
    #[error("output count must be between one and four")]
    InvalidOutputCount,
    #[error("provider attempt limit is invalid")]
    InvalidProviderAttempts,
    #[error("repair limit is invalid")]
    InvalidRepairs,
    #[error("required value is missing")]
    MissingRequiredValue,
}
