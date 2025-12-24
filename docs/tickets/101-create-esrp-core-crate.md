# Ticket 101: Create esrp-core Crate

**Phase:** 1 - Core Protocol Types
**Priority:** Critical (Blocking)
**Dependencies:** [001 - Create Cargo Workspace](001-create-cargo-workspace.md)
**Blocked By:** Phase 0 completion

## Summary

Set up the `esrp-core` crate with minimal dependencies. This crate contains all ESRP type definitions and validation logic with zero external dependencies beyond serde, uuid, and chrono.

## Context

The `esrp-core` crate is the foundation of the ESRP implementation. It defines all protocol types and must remain dependency-light to be easily embedded in other projects. All other ESRP crates depend on this one.

## Tasks

1. Update `crates/esrp-core/Cargo.toml` with required dependencies
2. Create module structure
3. Add crate-level documentation

## Implementation Details

### Cargo.toml

Update `crates/esrp-core/Cargo.toml`:

```toml
[package]
name = "esrp-core"
version.workspace = true
edition.workspace = true
description = "Core types for the Erasmus Service Request Protocol"
keywords = ["esrp", "protocol", "ai", "orchestration"]
categories = ["data-structures", "encoding"]

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
pretty_assertions = "1.4"
```

### Module Structure

Create the following file structure:

```
crates/esrp-core/src/
├── lib.rs           # Crate root, re-exports
├── types.rs         # All ESRP types (Ticket 102)
├── version.rs       # Version parsing and validation (Ticket 103)
├── validation.rs    # Request/response validation (Ticket 104)
└── error.rs         # Error types
```

### lib.rs

```rust
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
//! ```rust
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
pub use types::*;
pub use validation::*;
pub use version::*;
```

### error.rs (Skeleton)

```rust
//! Error types for ESRP Core

use thiserror::Error;

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
```

## Acceptance Criteria

- [ ] `cargo build --package esrp-core` compiles
- [ ] Module structure created with skeleton files
- [ ] Crate documentation added
- [ ] Only uses workspace dependencies (serde, uuid, chrono, thiserror)

## Verification

```bash
cargo build --package esrp-core
cargo doc --package esrp-core --open
```

## Notes

- Keep dependencies minimal - this crate should be embeddable anywhere
- Use `thiserror` for error types instead of manual `impl Error`
- All types will be implemented in subsequent tickets
- The `serde_json` dependency is needed for `serde_json::Value` in params
