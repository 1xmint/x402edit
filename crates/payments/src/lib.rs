use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use x402edit_domain::{Money, QuoteId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authorization {
    pub quote_id: QuoteId,
    pub opaque_signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedAuthorization {
    pub authorization_id: String,
    pub maximum: Money,
    pub expires_at_unix: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettlementReceipt {
    pub transaction_id: String,
    pub amount: Money,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("payment authorization was rejected")]
    Rejected,
    #[error("settlement outcome is unknown and requires reconciliation")]
    OutcomeUnknown,
    #[error("payment edge is unavailable")]
    Unavailable,
}

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn verify(
        &self,
        authorization: &Authorization,
        expected: &Money,
    ) -> Result<VerifiedAuthorization, PaymentError>;
    async fn settle(
        &self,
        authorization_id: &str,
        final_amount: &Money,
    ) -> Result<SettlementReceipt, PaymentError>;
}

/// Safe production default: paid work cannot start until a real edge is configured.
pub struct DisabledPaymentGateway;

#[async_trait]
impl PaymentGateway for DisabledPaymentGateway {
    async fn verify(
        &self,
        _authorization: &Authorization,
        _expected: &Money,
    ) -> Result<VerifiedAuthorization, PaymentError> {
        Err(PaymentError::Unavailable)
    }

    async fn settle(
        &self,
        _authorization_id: &str,
        _final_amount: &Money,
    ) -> Result<SettlementReceipt, PaymentError> {
        Err(PaymentError::Unavailable)
    }
}
