//! # ESRP Canonical
//!
//! Deterministic JSON serialization and hashing for the Erasmus Service Request Protocol.
//!
//! This crate provides:
//! - Canonical JSON serialization with sorted keys
//! - SHA256 hashing for content verification
//! - Payload hash derivation for idempotency
//!
//! ## Canonical JSON Rules
//!
//! 1. Object keys sorted lexicographically by UTF-8 bytes
//! 2. Arrays preserve insertion order
//! 3. No whitespace
//! 4. UTF-8 encoding
//! 5. **Floats are NOT allowed** - use strings
//!
//! ## Example
//!
//! ```rust
//! use esrp_canonical::{to_canonical_json_string, hash_canonical};
//!
//! // Canonicalize JSON
//! let value = serde_json::json!({"b": 1, "a": 2});
//! let canonical = to_canonical_json_string(&value).unwrap();
//! // Result: {"a":2,"b":1}
//!
//! // Hash content
//! let hash = hash_canonical(&value).unwrap();
//! // Result: 64-character hex string
//! ```
//!
//! ## Float Prohibition
//!
//! Floats are prohibited in canonical JSON regions because different platforms
//! serialize them inconsistently. Use strings instead:
//!
//! ```json
//! // WRONG - will cause hash mismatches
//! {"temperature": 0.7}
//!
//! // CORRECT - deterministic across platforms
//! {"temperature": "0.7"}
//! ```

mod canonical;
mod error;
mod hash;
mod payload;

pub use canonical::*;
pub use error::*;
pub use hash::*;
pub use payload::*;
