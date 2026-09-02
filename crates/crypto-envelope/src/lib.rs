#![forbid(unsafe_code)]

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hpke::{
    Deserializable, Kem as KemTrait, OpModeS, Serializable, aead::AesGcm128, kdf::HkdfSha256,
    kem::X25519HkdfSha256, setup_sender,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type Kem = X25519HkdfSha256;
type Kdf = HkdfSha256;
type Aead = AesGcm128;
pub const HPKE_SUITE: &str = "DHKEM(X25519,HKDF-SHA256)/HKDF-SHA256/AES-128-GCM";
pub const CHUNK_SIZE: usize = 1024 * 1024;
const INFO: &[u8] = b"x402edit/result-envelope/v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeManifest {
    pub schema_version: String,
    pub suite: String,
    pub encapsulated_key: String,
    pub media_type: String,
    pub plaintext_size: u64,
    pub ciphertext_sha256: String,
    pub chunk_size: usize,
    pub encrypted_chunk_sizes: Vec<u32>,
    pub associated_data_sha256: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("recipient public key is not valid X25519 key material")]
    InvalidRecipientKey,
    #[error("plaintext is too large")]
    PlaintextTooLarge,
    #[error("HPKE operation failed")]
    Hpke,
}

/// Produces sequentially authenticated HPKE records. Associated data is extended
/// with the big-endian chunk number to make reordering fail authentication.
pub fn seal(
    recipient_key: &[u8],
    plaintext: &[u8],
    associated_data: &[u8],
    media_type: impl Into<String>,
) -> Result<(EnvelopeManifest, Vec<u8>), EnvelopeError> {
    let public_key = <Kem as KemTrait>::PublicKey::from_bytes(recipient_key)
        .map_err(|_| EnvelopeError::InvalidRecipientKey)?;
    let (encapped_key, mut context) =
        setup_sender::<Aead, Kdf, Kem>(&OpModeS::Base, &public_key, INFO)
            .map_err(|_| EnvelopeError::Hpke)?;
    let mut ciphertext =
        Vec::with_capacity(plaintext.len() + 16 * plaintext.len().div_ceil(CHUNK_SIZE));
    let mut encrypted_chunk_sizes = Vec::new();
    for (index, chunk) in plaintext.chunks(CHUNK_SIZE).enumerate() {
        let sealed = context
            .seal(chunk, &chunk_aad(associated_data, index as u64))
            .map_err(|_| EnvelopeError::Hpke)?;
        encrypted_chunk_sizes
            .push(u32::try_from(sealed.len()).map_err(|_| EnvelopeError::PlaintextTooLarge)?);
        ciphertext.extend_from_slice(&sealed);
    }
    let manifest = EnvelopeManifest {
        schema_version: "1".into(),
        suite: HPKE_SUITE.into(),
        encapsulated_key: URL_SAFE_NO_PAD.encode(encapped_key.to_bytes()),
        media_type: media_type.into(),
        plaintext_size: u64::try_from(plaintext.len())
            .map_err(|_| EnvelopeError::PlaintextTooLarge)?,
        ciphertext_sha256: sha256_hex(&ciphertext),
        chunk_size: CHUNK_SIZE,
        encrypted_chunk_sizes,
        associated_data_sha256: sha256_hex(associated_data),
    };
    Ok((manifest, ciphertext))
}

fn chunk_aad(base: &[u8], index: u64) -> Vec<u8> {
    let mut value = Vec::with_capacity(base.len() + 8);
    value.extend_from_slice(base);
    value.extend_from_slice(&index.to_be_bytes());
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use hpke::{OpModeR, setup_receiver};

    #[test]
    fn round_trip_and_tamper_detection() {
        let (secret, public) = Kem::gen_keypair();
        let plaintext = vec![42_u8; CHUNK_SIZE + 7];
        let aad = b"job_1|q_1|1|application/vnd.x402edit.bundle+zip";
        let (manifest, ciphertext) = seal(
            public.to_bytes().as_slice(),
            &plaintext,
            aad,
            "application/vnd.x402edit.bundle+zip",
        )
        .unwrap();
        assert_eq!(manifest.encrypted_chunk_sizes.len(), 2);
        let encapped_bytes = URL_SAFE_NO_PAD.decode(&manifest.encapsulated_key).unwrap();
        let encapped = <Kem as KemTrait>::EncappedKey::from_bytes(&encapped_bytes).unwrap();
        let mut receiver =
            setup_receiver::<Aead, Kdf, Kem>(&OpModeR::Base, &secret, &encapped, INFO).unwrap();
        let mut offset = 0;
        let mut opened = Vec::new();
        for (index, size) in manifest.encrypted_chunk_sizes.iter().enumerate() {
            let end = offset + *size as usize;
            opened.extend(
                receiver
                    .open(&ciphertext[offset..end], &chunk_aad(aad, index as u64))
                    .unwrap(),
            );
            offset = end;
        }
        assert_eq!(opened, plaintext);
        let mut tampered = ciphertext;
        tampered[0] ^= 1;
        let mut receiver =
            setup_receiver::<Aead, Kdf, Kem>(&OpModeR::Base, &secret, &encapped, INFO).unwrap();
        assert!(
            receiver
                .open(
                    &tampered[..manifest.encrypted_chunk_sizes[0] as usize],
                    &chunk_aad(aad, 0)
                )
                .is_err()
        );
    }
}
