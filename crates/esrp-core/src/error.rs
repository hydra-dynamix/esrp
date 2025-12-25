//! Error types for ESRP Core

use thiserror::Error;

use crate::validation::ValidationError;
use crate::version::VersionError;

/// Errors that can occur during ESRP operations
#[derive(Debug, Error)]
pub enum ESRPError {
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
