//! Error types for ESRP-Trust.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrustError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Attestation not found: {0}")]
    AttestationNotFound(String),

    #[error("Chain validation failed: {0}")]
    ChainValidationFailed(String),

    #[error("Revoked attestation: {0}")]
    RevokedAttestation(String),

    #[error("Insufficient attestations: need {required}, found {found}")]
    InsufficientAttestations { required: usize, found: usize },

    #[error("Subject hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
