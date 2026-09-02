#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub const HPKE_SUITE: &str = "DHKEM(X25519,HKDF-SHA256)/HKDF-SHA256/AES-128-GCM";
pub const CHUNK_SIZE: usize = 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeManifest {
    pub suite: String,
    pub encapsulated_key: String,
    pub media_type: String,
    pub ciphertext_sha256: String,
    pub chunk_size: usize,
    pub chunk_count: usize,
    pub associated_data_sha256: String,
}
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("recipient public key must decode to exactly 32 bytes")]
    InvalidRecipientKey,
    #[error("HPKE sealing is unavailable in this build")]
    HpkeUnavailable,
}
/// Fail closed until the reviewed RFC 9180 implementation is enabled.
pub fn seal(
    _recipient_key: &[u8],
    _plaintext: &[u8],
    _associated_data: &[u8],
) -> Result<(EnvelopeManifest, Vec<u8>), EnvelopeError> {
    Err(EnvelopeError::HpkeUnavailable)
}
