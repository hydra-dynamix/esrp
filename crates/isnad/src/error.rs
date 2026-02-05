//! Error types for Isnad.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IsnadError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

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

    #[error("AI CAPTCHA verification failed: {0}")]
    CaptchaFailed(String),
}

// Keep the old name as an alias for backwards compatibility
pub type TrustError = IsnadError;
