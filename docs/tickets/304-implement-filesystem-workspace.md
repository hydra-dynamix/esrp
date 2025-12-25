# Ticket 304: Implement FilesystemWorkspace

**Phase:** 3 - Workspace Semantics
**Priority:** High
**Dependencies:** [303 - Define WorkspaceProvider Trait](303-define-workspace-provider-trait.md)
**Blocked By:** Ticket 303

## Summary

Implement `FilesystemWorkspace` in `esrp-workspace/src/filesystem.rs`. This provides the default storage backend using the local filesystem.

## Context

`FilesystemWorkspace` is the primary storage backend for local development and single-server deployments. It:
- Maps workspace URIs to filesystem paths
- Stores files with content-addressed naming
- Verifies file integrity via SHA256
- Creates directories as needed

## Tasks

1. Implement `FilesystemWorkspace` struct
2. Implement `WorkspaceProvider` trait
3. Implement atomic file writes
4. Handle directory creation
5. Write comprehensive tests

## Implementation Details

### filesystem.rs

```rust
//! Filesystem implementation of WorkspaceProvider

use crate::error::WorkspaceError;
use crate::provider::WorkspaceProvider;
use crate::uri::WorkspaceUri;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Filesystem-based workspace provider
///
/// Maps workspace URIs to filesystem paths:
/// ```text
/// workspace://namespace/path -> base_dir/namespace/path
/// ```
///
/// # Example
///
/// ```rust
/// use esrp_workspace::{FilesystemWorkspace, WorkspaceProvider, WorkspaceUri};
///
/// let workspace = FilesystemWorkspace::new("/tmp/esrp-workspace");
///
/// // Store data
/// let uri = workspace.store("artifacts", b"Hello, world!")?;
///
/// // Retrieve data
/// let data = workspace.retrieve(&uri)?;
/// assert_eq!(data, b"Hello, world!");
/// ```
#[derive(Debug, Clone)]
pub struct FilesystemWorkspace {
    /// Base directory for all workspace storage
    base_dir: PathBuf,
}

impl FilesystemWorkspace {
    /// Create a new filesystem workspace
    ///
    /// The base directory will be created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The root directory for workspace storage
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Ensure a directory exists
    fn ensure_dir(&self, path: &Path) -> Result<(), WorkspaceError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                WorkspaceError::IoError(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }
        Ok(())
    }

    /// Generate a content-addressed filename
    fn content_filename(data: &[u8], extension: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // Use first 16 characters of hash for filename
        let prefix: String = hash.iter().take(8).map(|b| format!("{:02x}", b)).collect();

        match extension {
            Some(ext) => format!("{}.{}", prefix, ext),
            None => format!("{}.bin", prefix),
        }
    }

    /// Compute SHA256 hash of data
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Write data atomically (write to temp, then rename)
    fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<(), WorkspaceError> {
        self.ensure_dir(path)?;

        // Write to temporary file
        let temp_name = format!(".tmp.{}", Uuid::new_v4());
        let temp_path = path.with_file_name(&temp_name);

        let mut file = fs::File::create(&temp_path).map_err(|e| {
            WorkspaceError::IoError(format!("Failed to create temp file {:?}: {}", temp_path, e))
        })?;

        file.write_all(data).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            WorkspaceError::IoError(format!("Failed to write data: {}", e))
        })?;

        file.sync_all().map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            WorkspaceError::IoError(format!("Failed to sync file: {}", e))
        })?;

        // Rename to final path
        fs::rename(&temp_path, path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            WorkspaceError::IoError(format!("Failed to rename {:?} to {:?}: {}", temp_path, path, e))
        })?;

        Ok(())
    }
}

impl WorkspaceProvider for FilesystemWorkspace {
    fn resolve(&self, uri: &WorkspaceUri) -> Result<PathBuf, WorkspaceError> {
        let path = self.base_dir.join(&uri.namespace).join(&uri.path);
        Ok(path)
    }

    fn store(&self, namespace: &str, data: &[u8]) -> Result<WorkspaceUri, WorkspaceError> {
        // Validate namespace
        let _ = WorkspaceUri::new(namespace, "placeholder")?;

        // Generate content-addressed filename
        let filename = Self::content_filename(data, None);

        // Create URI
        let uri = WorkspaceUri::new(namespace, &filename)?;

        // Resolve to path
        let path = self.resolve(&uri)?;

        // Write atomically
        self.atomic_write(&path, data)?;

        Ok(uri)
    }

    fn store_at(&self, uri: &WorkspaceUri, data: &[u8]) -> Result<(), WorkspaceError> {
        let path = self.resolve(uri)?;
        self.atomic_write(&path, data)
    }

    fn retrieve(&self, uri: &WorkspaceUri) -> Result<Vec<u8>, WorkspaceError> {
        let path = self.resolve(uri)?;

        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::NotFound(path)
            } else {
                WorkspaceError::IoError(format!("Failed to read {:?}: {}", path, e))
            }
        })
    }

    fn exists(&self, uri: &WorkspaceUri) -> Result<bool, WorkspaceError> {
        let path = self.resolve(uri)?;
        Ok(path.exists())
    }

    fn verify(&self, uri: &WorkspaceUri, expected_sha256: &str) -> Result<bool, WorkspaceError> {
        let data = self.retrieve(uri)?;
        let actual = Self::compute_hash(&data);

        // Case-insensitive comparison
        Ok(actual.eq_ignore_ascii_case(expected_sha256))
    }

    fn delete(&self, uri: &WorkspaceUri) -> Result<(), WorkspaceError> {
        let path = self.resolve(uri)?;

        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Not found is OK
            Err(e) => Err(WorkspaceError::IoError(format!(
                "Failed to delete {:?}: {}",
                path, e
            ))),
        }
    }

    fn size(&self, uri: &WorkspaceUri) -> Result<u64, WorkspaceError> {
        let path = self.resolve(uri)?;

        let metadata = fs::metadata(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::NotFound(path)
            } else {
                WorkspaceError::IoError(format!("Failed to get metadata: {}", e))
            }
        })?;

        Ok(metadata.len())
    }

    fn hash(&self, uri: &WorkspaceUri) -> Result<String, WorkspaceError> {
        let data = self.retrieve(uri)?;
        Ok(Self::compute_hash(&data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_workspace() -> (TempDir, FilesystemWorkspace) {
        let dir = TempDir::new().unwrap();
        let workspace = FilesystemWorkspace::new(dir.path());
        (dir, workspace)
    }

    mod store_retrieve {
        use super::*;

        #[test]
        fn test_store_and_retrieve() {
            let (_dir, workspace) = temp_workspace();

            let data = b"Hello, world!";
            let uri = workspace.store("artifacts", data).unwrap();

            let retrieved = workspace.retrieve(&uri).unwrap();
            assert_eq!(retrieved, data);
        }

        #[test]
        fn test_store_creates_directory() {
            let (_dir, workspace) = temp_workspace();

            let data = b"test";
            let uri = workspace.store("new-namespace", data).unwrap();

            assert!(workspace.exists(&uri).unwrap());
        }

        #[test]
        fn test_store_at_specific_path() {
            let (_dir, workspace) = temp_workspace();

            let uri = WorkspaceUri::new("artifacts", "specific.txt").unwrap();
            let data = b"specific data";

            workspace.store_at(&uri, data).unwrap();

            let retrieved = workspace.retrieve(&uri).unwrap();
            assert_eq!(retrieved, data);
        }

        #[test]
        fn test_retrieve_not_found() {
            let (_dir, workspace) = temp_workspace();

            let uri = WorkspaceUri::new("artifacts", "nonexistent.txt").unwrap();
            let result = workspace.retrieve(&uri);

            assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
        }
    }

    mod exists_delete {
        use super::*;

        #[test]
        fn test_exists() {
            let (_dir, workspace) = temp_workspace();

            let uri = workspace.store("temp", b"data").unwrap();
            assert!(workspace.exists(&uri).unwrap());

            let fake = WorkspaceUri::new("temp", "fake.txt").unwrap();
            assert!(!workspace.exists(&fake).unwrap());
        }

        #[test]
        fn test_delete() {
            let (_dir, workspace) = temp_workspace();

            let uri = workspace.store("temp", b"data").unwrap();
            assert!(workspace.exists(&uri).unwrap());

            workspace.delete(&uri).unwrap();
            assert!(!workspace.exists(&uri).unwrap());
        }

        #[test]
        fn test_delete_nonexistent_ok() {
            let (_dir, workspace) = temp_workspace();

            let uri = WorkspaceUri::new("temp", "nonexistent.txt").unwrap();
            // Should not error
            workspace.delete(&uri).unwrap();
        }
    }

    mod verification {
        use super::*;

        #[test]
        fn test_verify_correct_hash() {
            let (_dir, workspace) = temp_workspace();

            let data = b"verify me";
            let uri = workspace.store("artifacts", data).unwrap();
            let hash = workspace.hash(&uri).unwrap();

            assert!(workspace.verify(&uri, &hash).unwrap());
        }

        #[test]
        fn test_verify_wrong_hash() {
            let (_dir, workspace) = temp_workspace();

            let data = b"verify me";
            let uri = workspace.store("artifacts", data).unwrap();

            let wrong_hash = "a".repeat(64);
            assert!(!workspace.verify(&uri, &wrong_hash).unwrap());
        }

        #[test]
        fn test_hash() {
            let (_dir, workspace) = temp_workspace();

            let data = b"";
            let uri = workspace.store("artifacts", data).unwrap();
            let hash = workspace.hash(&uri).unwrap();

            // Known hash of empty data
            assert_eq!(
                hash,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
        }
    }

    mod size {
        use super::*;

        #[test]
        fn test_size() {
            let (_dir, workspace) = temp_workspace();

            let data = b"12345";
            let uri = workspace.store("artifacts", data).unwrap();

            assert_eq!(workspace.size(&uri).unwrap(), 5);
        }
    }

    mod content_addressing {
        use super::*;

        #[test]
        fn test_same_content_same_filename() {
            let data = b"same content";
            let f1 = FilesystemWorkspace::content_filename(data, None);
            let f2 = FilesystemWorkspace::content_filename(data, None);

            assert_eq!(f1, f2);
        }

        #[test]
        fn test_different_content_different_filename() {
            let f1 = FilesystemWorkspace::content_filename(b"content 1", None);
            let f2 = FilesystemWorkspace::content_filename(b"content 2", None);

            assert_ne!(f1, f2);
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn test_resolve_path() {
            let dir = TempDir::new().unwrap();
            let workspace = FilesystemWorkspace::new(dir.path());

            let uri = WorkspaceUri::new("artifacts", "audio.wav").unwrap();
            let path = workspace.resolve(&uri).unwrap();

            assert_eq!(path, dir.path().join("artifacts").join("audio.wav"));
        }

        #[test]
        fn test_resolve_nested_path() {
            let dir = TempDir::new().unwrap();
            let workspace = FilesystemWorkspace::new(dir.path());

            let uri = WorkspaceUri::new("runs", "a/b/c.txt").unwrap();
            let path = workspace.resolve(&uri).unwrap();

            assert_eq!(path, dir.path().join("runs").join("a/b/c.txt"));
        }
    }
}
```

## Acceptance Criteria

- [ ] Can store and retrieve blobs
- [ ] Content-addressed filenames generated
- [ ] Atomic writes (temp file + rename)
- [ ] Directories created automatically
- [ ] SHA256 verification works
- [ ] Correct hash for known inputs (empty string)
- [ ] Delete handles nonexistent files gracefully
- [ ] Namespace isolation works
- [ ] All tests pass

## Verification

```bash
cargo test --package esrp-workspace filesystem
```

## Notes

- Use `tempfile` crate for test isolation
- Atomic writes prevent corruption from crashes
- Content-addressed naming enables deduplication
- Case-insensitive hash comparison for robustness
- Consider file locking for concurrent access in future
