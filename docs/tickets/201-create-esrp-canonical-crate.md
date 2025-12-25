# Ticket 201: Create esrp-canonical Crate

**Phase:** 2 - Canonical Representation
**Priority:** Critical (Blocking)
**Dependencies:** [101 - Create esrp-core Crate](101-create-esrp-core-crate.md)
**Blocked By:** Phase 1 completion

## Summary

Set up the `esrp-canonical` crate for deterministic JSON serialization and hashing. This crate is critical for ensuring cross-platform hash consistency.

## Context

Canonical JSON is essential for:
- Computing `payload_hash` for idempotency
- Deriving `idempotency_key` for deduplication
- Artifact verification via SHA256
- Event log integrity

The crate must produce byte-identical output across all platforms and implementations.

## Tasks

1. Update `crates/esrp-canonical/Cargo.toml` with dependencies
2. Create module structure
3. Add crate-level documentation

## Implementation Details

### Cargo.toml

Update `crates/esrp-canonical/Cargo.toml`:

```toml
[package]
name = "esrp-canonical"
version.workspace = true
edition.workspace = true
description = "Canonical JSON serialization for ESRP"
keywords = ["esrp", "canonical", "json", "hashing"]
categories = ["encoding", "cryptography"]

[dependencies]
esrp-core = { path = "../esrp-core" }
serde = { workspace = true }
serde_json = { workspace = true }
sha2 = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
hex = "0.4"
pretty_assertions = "1.4"
```

### Module Structure

```
crates/esrp-canonical/src/
├── lib.rs           # Crate root, public API
├── canonical.rs     # Canonical JSON implementation
├── hash.rs          # SHA256 hashing
├── payload.rs       # Payload hash derivation
└── error.rs         # Error types
```

### lib.rs

```rust
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
//! use esrp_canonical::{to_canonical_json, hash_canonical, derive_payload_hash};
//! use esrp_core::{Target, Input};
//!
//! // Canonicalize JSON
//! let value = serde_json::json!({"b": 1, "a": 2});
//! let canonical = to_canonical_json(&value)?;
//! // Result: {"a":2,"b":1}
//!
//! // Hash content
//! let hash = hash_canonical(&value)?;
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
```

### error.rs

```rust
//! Error types for ESRP Canonical

use thiserror::Error;

/// Errors that can occur during canonicalization or hashing
#[derive(Debug, Error, Clone, PartialEq)]
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
```

## Acceptance Criteria

- [ ] `cargo build --package esrp-canonical` compiles
- [ ] Module structure created
- [ ] Depends on `esrp-core`, `serde`, `serde_json`, `sha2`
- [ ] Crate documentation explains float prohibition
- [ ] Error types defined

## Verification

```bash
cargo build --package esrp-canonical
cargo doc --package esrp-canonical --open
```

## Notes

- This crate is the reference implementation for canonical JSON
- All language bindings must produce byte-identical output
- Float detection is critical - must reject, not silently convert
- Consider using `serde_json::Number::is_f64()` for detection
