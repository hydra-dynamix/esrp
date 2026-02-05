//! Workspace URI parsing and validation

use crate::error::WorkspaceError;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Maximum namespace length
pub const MAX_NAMESPACE_LENGTH: usize = 64;

/// Maximum path length
pub const MAX_PATH_LENGTH: usize = 1024;

/// Reserved namespace names
pub const RESERVED_NAMESPACES: &[&str] = &["system", "tmp", "cache"];

/// Workspace URI prefix
pub const WORKSPACE_URI_PREFIX: &str = "workspace://";

/// A parsed workspace URI
///
/// Workspace URIs have the format:
/// ```text
/// workspace://<namespace>/<path>
/// ```
///
/// # Examples
///
/// ```rust
/// use esrp_workspace::WorkspaceUri;
///
/// let uri = WorkspaceUri::parse("workspace://artifacts/audio.wav").unwrap();
/// assert_eq!(uri.namespace, "artifacts");
/// assert_eq!(uri.path.to_str(), Some("audio.wav"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceUri {
    /// The namespace (e.g., "artifacts", "temp")
    pub namespace: String,

    /// The path within the namespace
    pub path: PathBuf,
}

impl WorkspaceUri {
    /// Create a new workspace URI
    ///
    /// # Errors
    ///
    /// Returns an error if the namespace or path is invalid.
    pub fn new(
        namespace: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<Self, WorkspaceError> {
        let namespace = namespace.into();
        let path = path.into();

        Self::validate_namespace(&namespace)?;
        Self::validate_path(&path)?;

        Ok(Self { namespace, path })
    }

    /// Parse a workspace URI from a string
    ///
    /// # Format
    ///
    /// ```text
    /// workspace://<namespace>/<path>
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - URI doesn't start with `workspace://`
    /// - Namespace is invalid
    /// - Path is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use esrp_workspace::WorkspaceUri;
    ///
    /// // Valid URIs
    /// let uri = WorkspaceUri::parse("workspace://artifacts/audio.wav").unwrap();
    /// let uri = WorkspaceUri::parse("workspace://temp/subdir/file.txt").unwrap();
    ///
    /// // Invalid URIs
    /// assert!(WorkspaceUri::parse("file://test").is_err());
    /// assert!(WorkspaceUri::parse("workspace://temp/../etc/passwd").is_err());
    /// ```
    pub fn parse(uri: &str) -> Result<Self, WorkspaceError> {
        // Check prefix
        if !uri.starts_with(WORKSPACE_URI_PREFIX) {
            return Err(WorkspaceError::InvalidUri(format!(
                "URI must start with '{}', got: {}",
                WORKSPACE_URI_PREFIX, uri
            )));
        }

        // Remove prefix
        let rest = &uri[WORKSPACE_URI_PREFIX.len()..];

        if rest.is_empty() {
            return Err(WorkspaceError::InvalidUri(
                "URI must contain namespace and path".to_string(),
            ));
        }

        // Split into namespace and path
        let (namespace, path) = match rest.find('/') {
            Some(idx) => {
                let ns = &rest[..idx];
                let path = &rest[idx + 1..];
                (ns, path)
            }
            None => {
                // No path, just namespace
                return Err(WorkspaceError::InvalidUri(
                    "URI must contain both namespace and path".to_string(),
                ));
            }
        };

        if path.is_empty() {
            return Err(WorkspaceError::InvalidUri(
                "Path cannot be empty".to_string(),
            ));
        }

        Self::new(namespace, path)
    }

    /// Check if the namespace is reserved
    pub fn is_reserved_namespace(&self) -> bool {
        RESERVED_NAMESPACES.contains(&self.namespace.as_str())
    }

    /// Validate a namespace string
    fn validate_namespace(namespace: &str) -> Result<(), WorkspaceError> {
        if namespace.is_empty() {
            return Err(WorkspaceError::InvalidNamespace(
                namespace.to_string(),
                "Namespace cannot be empty".to_string(),
            ));
        }

        if namespace.len() > MAX_NAMESPACE_LENGTH {
            return Err(WorkspaceError::NamespaceTooLong(namespace.to_string()));
        }

        // Check character set: [a-zA-Z0-9._-]
        for c in namespace.chars() {
            if !c.is_ascii_alphanumeric() && c != '.' && c != '_' && c != '-' {
                return Err(WorkspaceError::InvalidNamespace(
                    namespace.to_string(),
                    format!("Invalid character '{}'. Allowed: a-z, A-Z, 0-9, ., _, -", c),
                ));
            }
        }

        Ok(())
    }

    /// Validate a path
    fn validate_path(path: &Path) -> Result<(), WorkspaceError> {
        let path_str = path.to_string_lossy();

        if path_str.is_empty() {
            return Err(WorkspaceError::InvalidPath(
                path_str.to_string(),
                "Path cannot be empty".to_string(),
            ));
        }

        if path_str.len() > MAX_PATH_LENGTH {
            return Err(WorkspaceError::PathTooLong(path_str.to_string()));
        }

        // Check for path traversal
        if path_str.contains("..") {
            return Err(WorkspaceError::PathTraversal(path_str.to_string()));
        }

        // Check for absolute path
        if path.is_absolute() || path_str.starts_with('/') || path_str.starts_with('\\') {
            return Err(WorkspaceError::InvalidPath(
                path_str.to_string(),
                "Path must be relative (no leading / or \\)".to_string(),
            ));
        }

        // Check for null bytes
        if path_str.contains('\0') {
            return Err(WorkspaceError::InvalidPath(
                path_str.to_string(),
                "Path cannot contain null bytes".to_string(),
            ));
        }

        Ok(())
    }
}

impl Display for WorkspaceUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}/{}",
            WORKSPACE_URI_PREFIX,
            self.namespace,
            // Use forward slashes for URIs
            self.path.to_string_lossy().replace('\\', "/")
        )
    }
}

impl FromStr for WorkspaceUri {
    type Err = WorkspaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parsing {
        use super::*;

        #[test]
        fn test_parse_simple_uri() {
            let uri = WorkspaceUri::parse("workspace://artifacts/audio.wav").unwrap();
            assert_eq!(uri.namespace, "artifacts");
            assert_eq!(uri.path, PathBuf::from("audio.wav"));
        }

        #[test]
        fn test_parse_nested_path() {
            let uri = WorkspaceUri::parse("workspace://temp/subdir/file.txt").unwrap();
            assert_eq!(uri.namespace, "temp");
            assert_eq!(uri.path, PathBuf::from("subdir/file.txt"));
        }

        #[test]
        fn test_parse_deeply_nested() {
            let uri = WorkspaceUri::parse("workspace://runs/a/b/c/d/e/file.json").unwrap();
            assert_eq!(uri.namespace, "runs");
            assert_eq!(uri.path, PathBuf::from("a/b/c/d/e/file.json"));
        }

        #[test]
        fn test_invalid_prefix() {
            assert!(WorkspaceUri::parse("file://test/path").is_err());
            assert!(WorkspaceUri::parse("http://test/path").is_err());
            assert!(WorkspaceUri::parse("/absolute/path").is_err());
        }

        #[test]
        fn test_missing_path() {
            assert!(WorkspaceUri::parse("workspace://namespace").is_err());
            assert!(WorkspaceUri::parse("workspace://namespace/").is_err());
        }

        #[test]
        fn test_empty_namespace() {
            assert!(WorkspaceUri::parse("workspace:///path").is_err());
        }
    }

    mod namespace_validation {
        use super::*;

        #[test]
        fn test_valid_namespaces() {
            assert!(WorkspaceUri::parse("workspace://artifacts/f").is_ok());
            assert!(WorkspaceUri::parse("workspace://temp-files/f").is_ok());
            assert!(WorkspaceUri::parse("workspace://my_namespace/f").is_ok());
            assert!(WorkspaceUri::parse("workspace://data.v1/f").is_ok());
            assert!(WorkspaceUri::parse("workspace://UPPER/f").is_ok());
            assert!(WorkspaceUri::parse("workspace://Mix3d/f").is_ok());
        }

        #[test]
        fn test_invalid_namespace_characters() {
            assert!(WorkspaceUri::parse("workspace://with space/f").is_err());
            assert!(WorkspaceUri::parse("workspace://with:colon/f").is_err());
            assert!(WorkspaceUri::parse("workspace://with@symbol/f").is_err());
        }

        #[test]
        fn test_namespace_too_long() {
            let long_ns = "a".repeat(65);
            let uri = format!("workspace://{}/file", long_ns);
            assert!(matches!(
                WorkspaceUri::parse(&uri),
                Err(WorkspaceError::NamespaceTooLong(_))
            ));
        }

        #[test]
        fn test_max_length_namespace_ok() {
            let ns = "a".repeat(64);
            let uri = format!("workspace://{}/file", ns);
            assert!(WorkspaceUri::parse(&uri).is_ok());
        }
    }

    mod path_validation {
        use super::*;

        #[test]
        fn test_path_traversal_rejected() {
            assert!(matches!(
                WorkspaceUri::parse("workspace://temp/../etc/passwd"),
                Err(WorkspaceError::PathTraversal(_))
            ));
            assert!(matches!(
                WorkspaceUri::parse("workspace://temp/subdir/../secret"),
                Err(WorkspaceError::PathTraversal(_))
            ));
        }

        #[test]
        fn test_absolute_path_rejected() {
            // These would have the path start with /
            assert!(WorkspaceUri::parse("workspace://ns//absolute").is_err());
        }

        #[test]
        fn test_path_too_long() {
            let long_path = "a".repeat(1025);
            let uri = format!("workspace://ns/{}", long_path);
            assert!(matches!(
                WorkspaceUri::parse(&uri),
                Err(WorkspaceError::PathTooLong(_))
            ));
        }

        #[test]
        fn test_max_length_path_ok() {
            let path = "a".repeat(1024);
            let uri = format!("workspace://ns/{}", path);
            assert!(WorkspaceUri::parse(&uri).is_ok());
        }
    }

    mod display {
        use super::*;

        #[test]
        fn test_display_round_trip() {
            let original = "workspace://artifacts/audio.wav";
            let uri = WorkspaceUri::parse(original).unwrap();
            assert_eq!(uri.to_string(), original);
        }

        #[test]
        fn test_display_nested_path() {
            let original = "workspace://temp/a/b/c.txt";
            let uri = WorkspaceUri::parse(original).unwrap();
            assert_eq!(uri.to_string(), original);
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn test_from_str() {
            let uri: WorkspaceUri = "workspace://ns/path".parse().unwrap();
            assert_eq!(uri.namespace, "ns");
        }
    }

    mod reserved_namespaces {
        use super::*;

        #[test]
        fn test_reserved_namespace_detection() {
            let uri = WorkspaceUri::parse("workspace://system/file").unwrap();
            assert!(uri.is_reserved_namespace());

            let uri = WorkspaceUri::parse("workspace://tmp/file").unwrap();
            assert!(uri.is_reserved_namespace());

            let uri = WorkspaceUri::parse("workspace://cache/file").unwrap();
            assert!(uri.is_reserved_namespace());

            let uri = WorkspaceUri::parse("workspace://artifacts/file").unwrap();
            assert!(!uri.is_reserved_namespace());
        }
    }
}
