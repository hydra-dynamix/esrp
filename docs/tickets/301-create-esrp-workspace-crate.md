# Ticket 301: Create esrp-workspace Crate

**Phase:** 3 - Workspace Semantics
**Priority:** High
**Dependencies:** [101 - Create esrp-core Crate](101-create-esrp-core-crate.md)
**Blocked By:** Phase 1 completion

## Summary

Set up the `esrp-workspace` crate for handling workspace URIs and artifact storage. This crate provides the abstraction layer for file storage.

## Context

The workspace crate enables:
- Parsing and validating workspace URIs (`workspace://namespace/path`)
- Abstract storage through the `WorkspaceProvider` trait
- Filesystem-based implementation for local development
- Artifact verification via SHA256

## Tasks

1. Update `crates/esrp-workspace/Cargo.toml`
2. Create module structure
3. Add crate documentation

## Implementation Details

### Cargo.toml

Update `crates/esrp-workspace/Cargo.toml`:

```toml
[package]
name = "esrp-workspace"
version.workspace = true
edition.workspace = true
description = "Workspace URI handling and artifact storage for ESRP"
keywords = ["esrp", "workspace", "storage", "artifacts"]
categories = ["filesystem", "encoding"]

[dependencies]
thiserror = { workspace = true }
sha2 = { workspace = true }
uuid = { workspace = true }

[dev-dependencies]
tempfile = "3.0"
```

### Module Structure

```
crates/esrp-workspace/src/
├── lib.rs           # Crate root, public API
├── uri.rs           # Workspace URI parsing
├── provider.rs      # WorkspaceProvider trait
├── filesystem.rs    # Filesystem implementation
└── error.rs         # Error types
```

### lib.rs

```rust
//! # ESRP Workspace
//!
//! Workspace URI handling and artifact storage for the Erasmus Service Request Protocol.
//!
//! This crate provides:
//! - Workspace URI parsing and validation
//! - Abstract storage through `WorkspaceProvider` trait
//! - Filesystem implementation for local storage
//! - Artifact verification via SHA256
//!
//! ## Workspace URIs
//!
//! ESRP uses workspace URIs to reference artifacts:
//!
//! ```text
//! workspace://<namespace>/<path>
//! ```
//!
//! Examples:
//! - `workspace://artifacts/audio_12345.wav`
//! - `workspace://temp/image.png`
//! - `workspace://runs/run-abc/output.json`
//!
//! ## Example Usage
//!
//! ```rust
//! use esrp_workspace::{WorkspaceUri, FilesystemWorkspace, WorkspaceProvider};
//! use std::path::PathBuf;
//!
//! // Parse a workspace URI
//! let uri = WorkspaceUri::parse("workspace://artifacts/audio.wav")?;
//! assert_eq!(uri.namespace, "artifacts");
//! assert_eq!(uri.path.to_str(), Some("audio.wav"));
//!
//! // Create a filesystem workspace
//! let workspace = FilesystemWorkspace::new("/tmp/esrp");
//!
//! // Resolve URI to filesystem path
//! let path = workspace.resolve(&uri)?;
//! // Result: /tmp/esrp/artifacts/audio.wav
//!
//! // Store data and get URI
//! let data = b"Hello, world!";
//! let uri = workspace.store("artifacts", data)?;
//! ```
//!
//! ## Security
//!
//! The workspace implementation prevents:
//! - Path traversal attacks (`..` in paths)
//! - Absolute paths in URIs
//! - Invalid namespace characters

mod error;
mod filesystem;
mod provider;
mod uri;

pub use error::*;
pub use filesystem::*;
pub use provider::*;
pub use uri::*;
```

### error.rs

```rust
//! Error types for ESRP Workspace

use thiserror::Error;
use std::path::PathBuf;

/// Errors that can occur during workspace operations
#[derive(Debug, Error, Clone, PartialEq)]
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
```

## Acceptance Criteria

- [ ] `cargo build --package esrp-workspace` compiles
- [ ] Module structure created
- [ ] Minimal dependencies (thiserror, sha2, uuid)
- [ ] Crate documentation explains workspace concept
- [ ] Error types defined

## Verification

```bash
cargo build --package esrp-workspace
cargo doc --package esrp-workspace --open
```

## Notes

- Keep dependencies minimal
- No dependency on esrp-core to avoid circular dependencies
- The trait should be object-safe for flexibility
- Consider async trait in future for remote storage
