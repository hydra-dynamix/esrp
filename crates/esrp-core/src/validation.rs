//! Request and Response Validation
//!
//! This module provides validation logic for ESRP requests and responses.
//! Implementation will be completed in Ticket 104.

use thiserror::Error;

/// Errors that can occur during validation
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field}: {reason}")]
    InvalidValue { field: String, reason: String },
}
