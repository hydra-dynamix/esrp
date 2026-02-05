//! WorkspaceProvider trait definition

use crate::error::WorkspaceError;
use crate::uri::WorkspaceUri;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Trait for workspace storage providers
///
/// This trait abstracts the storage layer, enabling different backends:
/// - Filesystem (local development)
/// - S3/Object storage (production)
/// - In-memory (testing)
/// - Content-addressed store
///
/// # Object Safety
///
/// This trait is object-safe and can be used with `dyn WorkspaceProvider`.
pub trait WorkspaceProvider: Send + Sync {
    /// Resolve a workspace URI to a filesystem path
    ///
    /// This converts a `workspace://namespace/path` URI to an absolute
    /// filesystem path. The path may not exist yet.
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to resolve
    ///
    /// # Returns
    ///
    /// The absolute filesystem path for the URI.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI cannot be resolved (e.g., invalid namespace).
    fn resolve(&self, uri: &WorkspaceUri) -> Result<PathBuf, WorkspaceError>;

    /// Store data in the workspace and return a URI
    ///
    /// The provider generates a unique filename based on the content hash.
    /// The data is stored atomically (write-to-temp, then rename).
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace to store in
    /// * `data` - The data to store
    ///
    /// # Returns
    ///
    /// A workspace URI pointing to the stored data.
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails (disk full, permission denied, etc.).
    fn store(&self, namespace: &str, data: &[u8]) -> Result<WorkspaceUri, WorkspaceError>;

    /// Store data with a specific filename
    ///
    /// Unlike `store()`, this allows specifying the exact path.
    /// **Warning**: This may overwrite existing files.
    ///
    /// # Arguments
    ///
    /// * `uri` - The exact URI to store at
    /// * `data` - The data to store
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails.
    fn store_at(&self, uri: &WorkspaceUri, data: &[u8]) -> Result<(), WorkspaceError>;

    /// Retrieve data from the workspace
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to retrieve
    ///
    /// # Returns
    ///
    /// The data stored at the URI.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` if the file doesn't exist.
    fn retrieve(&self, uri: &WorkspaceUri) -> Result<Vec<u8>, WorkspaceError>;

    /// Check if a URI exists in the workspace
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to check
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    fn exists(&self, uri: &WorkspaceUri) -> Result<bool, WorkspaceError>;

    /// Verify that data at a URI matches the expected SHA256 hash
    ///
    /// This is critical for artifact verification. Always verify before
    /// using data from untrusted sources.
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to verify
    /// * `expected_sha256` - The expected SHA256 hash (64 hex characters)
    ///
    /// # Returns
    ///
    /// `true` if the hash matches, `false` if it doesn't.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` if the file doesn't exist.
    fn verify(&self, uri: &WorkspaceUri, expected_sha256: &str) -> Result<bool, WorkspaceError>;

    /// Delete data from the workspace
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to delete
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails (file in use, permission denied, etc.).
    /// Does NOT return error if file doesn't exist.
    fn delete(&self, uri: &WorkspaceUri) -> Result<(), WorkspaceError>;

    /// Get the size of data at a URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to check
    ///
    /// # Returns
    ///
    /// The size in bytes.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` if the file doesn't exist.
    fn size(&self, uri: &WorkspaceUri) -> Result<u64, WorkspaceError>;

    /// Compute the SHA256 hash of data at a URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The workspace URI to hash
    ///
    /// # Returns
    ///
    /// The SHA256 hash as a 64-character hex string.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` if the file doesn't exist.
    fn hash(&self, uri: &WorkspaceUri) -> Result<String, WorkspaceError>;
}

/// Extension trait for workspace providers with additional utilities
pub trait WorkspaceProviderExt: WorkspaceProvider {
    /// Store data and return the URI along with hash and size
    ///
    /// Useful for creating artifact records.
    fn store_with_metadata(
        &self,
        namespace: &str,
        data: &[u8],
    ) -> Result<StoredArtifact, WorkspaceError> {
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let sha256: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

        // Store
        let uri = self.store(namespace, data)?;

        Ok(StoredArtifact {
            uri,
            sha256,
            size_bytes: data.len() as u64,
        })
    }

    /// Retrieve and verify data in one operation
    fn retrieve_verified(
        &self,
        uri: &WorkspaceUri,
        expected_sha256: &str,
    ) -> Result<Vec<u8>, WorkspaceError> {
        let data = self.retrieve(uri)?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let actual: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        if actual != expected_sha256 {
            return Err(WorkspaceError::HashMismatch {
                expected: expected_sha256.to_string(),
                actual,
            });
        }

        Ok(data)
    }
}

// Blanket implementation
impl<T: WorkspaceProvider> WorkspaceProviderExt for T {}

/// Metadata for a stored artifact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredArtifact {
    /// The workspace URI
    pub uri: WorkspaceUri,

    /// SHA256 hash of the content
    pub sha256: String,

    /// Size in bytes
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify trait is object-safe
    fn _assert_object_safe(_: &dyn WorkspaceProvider) {}
}
