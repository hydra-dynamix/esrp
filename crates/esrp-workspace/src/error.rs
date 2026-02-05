//! Error types for ESRP Workspace

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during workspace operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    #[error("Invalid workspace URI: {0}")]
    InvalidUri(String),

    #[error("Invalid namespace '{0}': {1}")]
    InvalidNamespace(String, String),

    #[error("Invalid path '{0}': {1}")]
    InvalidPath(String, String),

    #[error("Path traversal not allowed: {0}")]
    PathTraversal(String),

    #[error("Namespace too long: {0} (max 64 characters)")]
    NamespaceTooLong(String),

    #[error("Path too long: {0} (max 1024 characters)")]
    PathTooLong(String),

    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Reserved namespace: {0}")]
    ReservedNamespace(String),
}
