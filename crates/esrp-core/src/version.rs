//! Version Parsing and Validation
//!
//! This module handles ESRP version parsing and compatibility checking.
//! Implementation will be completed in Ticket 103.

use thiserror::Error;

/// Errors that can occur during version parsing or validation
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
}
