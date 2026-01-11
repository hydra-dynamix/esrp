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
//! let uri = WorkspaceUri::parse("workspace://artifacts/audio.wav").unwrap();
//! assert_eq!(uri.namespace, "artifacts");
//! assert_eq!(uri.path.to_str(), Some("audio.wav"));
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
