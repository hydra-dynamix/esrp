//! Error types for ESRP Canonical

use thiserror::Error;

/// Errors that can occur during canonicalization or hashing
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CanonicalError {
    #[error("Float values are not allowed in canonical JSON. Use strings instead (e.g., \"0.7\" instead of 0.7)")]
    FloatNotAllowed,

    #[error("JSON serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid JSON structure: {0}")]
    InvalidStructure(String),
}

impl From<serde_json::Error> for CanonicalError {
    fn from(err: serde_json::Error) -> Self {
        CanonicalError::SerializationError(err.to_string())
    }
}
