//! # ESRP Core
//!
//! Core types and validation for the Erasmus Service Request Protocol.
//!
//! This crate provides:
//! - Type definitions for ESRP requests and responses
//! - Version parsing and compatibility checking
//! - Input validation
//!
//! ## Example
//!
//! ```rust,ignore
//! use esrp_core::{ESRPRequest, ESRPResponse, validate_request};
//!
//! // Parse a request
//! let request: ESRPRequest = serde_json::from_str(json)?;
//!
//! // Validate
//! validate_request(&request)?;
//! ```

pub mod error;
pub mod types;
pub mod validation;
pub mod version;

// Re-exports for convenience
pub use error::*;
// pub use types::*; // TODO: Uncomment when types are implemented (Ticket 102)
pub use validation::*;
pub use version::*;
