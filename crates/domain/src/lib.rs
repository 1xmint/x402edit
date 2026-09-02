#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub const SCHEMA_VERSION_V1: &str = "1";
pub const MAX_PROMPT_BYTES: usize = 16 * 1024;
pub const MAX_LITERAL_TEXT_ENTRIES: usize = 64;
pub const MAX_LITERAL_TEXT_BYTES: usize = 8 * 1024;
pub const MAX_REFERENCE_COUNT: u8 = 14;
pub const MAX_OUTPUT_COUNT: u8 = 4;
pub const MAX_PROVIDER_ATTEMPTS: u8 = 2;
pub const MAX_REPAIRS: u8 = 2;
pub const MAX_WALL_TIME_SECONDS: u32 = 900;

macro_rules! prefixed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}{}", $prefix, Uuid::now_v7()))
            }

            pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                let suffix = value
                    .strip_prefix($prefix)
                    .ok_or_else(|| DomainError::InvalidId(value.clone()))?;
                Uuid::parse_str(suffix).map_err(|_| DomainError::InvalidId(value.clone()))?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

prefixed_id!(QuoteId, "q_");
prefixed_id!(JobId, "job_");
prefixed_id!(RequestId, "req_");
prefixed_id!(ArtifactId, "art_");
prefixed_id!(VersionId, "ver_");
prefixed_id!(AttemptId, "att_");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Generate,
    Edit,
    Design,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VisualMode {
    Auto,
    Image,
    StructuredDesign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    StrictEphemeral,
    ProviderConsent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualityProfile {
    Quality,
    Balanced,
    Economy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    #[serde(rename = "0.5k")]
    HalfK,
    #[serde(rename = "1k")]
    OneK,
    #[serde(rename = "2k")]
    TwoK,
    #[serde(rename = "4k")]
    FourK,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    AwaitingInputs,
    NeedsInput,
    Queued,
    Running,
    PaymentPending,
    ReconciliationRequired,
    Ready,
    Failed,
    Cancelled,
    Expired,
    Purged,
}

impl JobState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use JobState::*;
        matches!(
            (self, next),
            (AwaitingInputs, NeedsInput | Queued | Cancelled | Expired)
                | (NeedsInput, Queued | Cancelled | Expired)
                | (Queued, Running | Cancelled | Expired)
                | (Running, PaymentPending | Failed)
                | (PaymentPending, Ready | ReconciliationRequired | Failed)
                | (ReconciliationRequired, Ready | Failed)
                | (Ready, Purged)
                | (Failed, Purged)
                | (Cancelled, Purged)
                | (Expired, Purged)
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Failed | Self::Cancelled | Self::Expired | Self::Purged
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobPhase {
    Intake,
    Planning,
    Routing,
    ProviderSubmit,
    ProviderWait,
    Ingest,
    Compose,
    Validate,
    Repair,
    Encrypt,
    Settle,
    Deliver,
    Purge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentState {
    Required,
    Verified,
    Authorized,
    SettlementRequested,
    SettlementUnknown,
    Settled,
    Voided,
    Expired,
}

impl PaymentState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use PaymentState::*;
        matches!(
            (self, next),
            (Required, Verified | Expired)
                | (Verified, Authorized | Voided | Expired)
                | (Authorized, SettlementRequested | Voided | Expired)
                | (SettlementRequested, Settled | SettlementUnknown)
                | (SettlementUnknown, Settled | Voided)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Money {
    pub asset: String,
    pub amount_atomic: u64,
    pub decimals: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobSnapshot {
    pub id: JobId,
    pub quote_id: QuoteId,
    pub operation: Operation,
    pub state: JobState,
    pub phase: JobPhase,
    pub payment_state: PaymentState,
    pub fencing_token: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result_expires_at: Option<DateTime<Utc>>,
    pub warnings: Vec<String>,
}

impl JobSnapshot {
    pub fn transition(
        &mut self,
        next_state: JobState,
        next_phase: JobPhase,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if !self.state.can_transition_to(next_state) {
            return Err(DomainError::InvalidJobTransition {
                from: self.state,
                to: next_state,
            });
        }
        self.state = next_state;
        self.phase = next_phase;
        self.updated_at = now;
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid prefixed UUID: {0}")]
    InvalidId(String),
    #[error("invalid job transition from {from:?} to {to:?}")]
    InvalidJobTransition { from: JobState, to: JobState },
    #[error("invalid payment transition from {from:?} to {to:?}")]
    InvalidPaymentTransition {
        from: PaymentState,
        to: PaymentState,
    },
    #[error("validation failed: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        let id = JobId::new();
        assert_eq!(JobId::parse(id.to_string()).unwrap(), id);
        assert!(JobId::parse("q_not-a-job").is_err());
    }

    #[test]
    fn illegal_job_transition_is_rejected() {
        assert!(!JobState::AwaitingInputs.can_transition_to(JobState::Ready));
        assert!(JobState::AwaitingInputs.can_transition_to(JobState::Queued));
        assert!(JobState::Ready.can_transition_to(JobState::Purged));
    }

    #[test]
    fn terminal_states_are_explicit() {
        assert!(JobState::Purged.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(!JobState::Ready.is_terminal());
    }
}
