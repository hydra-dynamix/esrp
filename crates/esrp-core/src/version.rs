//! ESRP Version handling
//!
//! This module handles ESRP version parsing and compatibility checking.
//! ESRP uses a `major.minor` version format (e.g., "1.0", "1.5", "2.0").
//!
//! Compatibility rules:
//! - Same major version = compatible
//! - Different major version = incompatible
//! - Minor version changes are backward compatible additions

use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

/// Current ESRP major version
pub const ESRP_MAJOR_VERSION: u8 = 1;

/// Current ESRP minor version
pub const ESRP_MINOR_VERSION: u8 = 0;

/// ESRP version string constant
pub const ESRP_VERSION: &str = "1.0";

/// Errors that can occur during version operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VersionError {
    #[error("Invalid version format: '{0}'. Expected 'major.minor' (e.g., '1.0')")]
    InvalidFormat(String),

    #[error("Invalid major version: '{0}'. Must be a non-negative integer")]
    InvalidMajor(String),

    #[error("Invalid minor version: '{0}'. Must be a non-negative integer")]
    InvalidMinor(String),

    #[error("Empty version string")]
    Empty,

    #[error("Version {got} is incompatible with {expected}. Major versions must match")]
    Incompatible { got: String, expected: String },

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
}

/// ESRP Version representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ESRPVersion {
    pub major: u8,
    pub minor: u8,
}

impl ESRPVersion {
    /// Create a new version
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Get the current protocol version
    pub fn current() -> Self {
        Self::new(ESRP_MAJOR_VERSION, ESRP_MINOR_VERSION)
    }

    /// Parse a version string
    ///
    /// # Examples
    ///
    /// ```
    /// use esrp_core::ESRPVersion;
    ///
    /// let v = ESRPVersion::parse("1.0").unwrap();
    /// assert_eq!(v.major, 1);
    /// assert_eq!(v.minor, 0);
    /// ```
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        if s.is_empty() {
            return Err(VersionError::Empty);
        }

        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 2 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse::<u8>()
            .map_err(|_| VersionError::InvalidMajor(parts[0].to_string()))?;

        let minor = parts[1]
            .parse::<u8>()
            .map_err(|_| VersionError::InvalidMinor(parts[1].to_string()))?;

        Ok(Self { major, minor })
    }

    /// Check if this version is compatible with another version
    ///
    /// Versions are compatible if they have the same major version.
    ///
    /// # Examples
    ///
    /// ```
    /// use esrp_core::ESRPVersion;
    ///
    /// let v1_0 = ESRPVersion::new(1, 0);
    /// let v1_5 = ESRPVersion::new(1, 5);
    /// let v2_0 = ESRPVersion::new(2, 0);
    ///
    /// assert!(v1_0.is_compatible_with(&v1_5));  // Same major
    /// assert!(!v1_0.is_compatible_with(&v2_0)); // Different major
    /// ```
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }

    /// Check compatibility and return an error if incompatible
    pub fn require_compatible(&self, other: &Self) -> Result<(), VersionError> {
        if self.is_compatible_with(other) {
            Ok(())
        } else {
            Err(VersionError::Incompatible {
                got: self.to_string(),
                expected: other.to_string(),
            })
        }
    }

    /// Check if this version is the current version
    pub fn is_current(&self) -> bool {
        *self == Self::current()
    }

    /// Check if a version string is compatible with the current version
    pub fn is_compatible_str(version_str: &str) -> Result<bool, VersionError> {
        let version = Self::parse(version_str)?;
        Ok(version.is_compatible_with(&Self::current()))
    }
}

impl Display for ESRPVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ESRPVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Default for ESRPVersion {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_versions() {
        assert_eq!(ESRPVersion::parse("1.0").unwrap(), ESRPVersion::new(1, 0));
        assert_eq!(ESRPVersion::parse("1.5").unwrap(), ESRPVersion::new(1, 5));
        assert_eq!(ESRPVersion::parse("2.0").unwrap(), ESRPVersion::new(2, 0));
        assert_eq!(ESRPVersion::parse("0.1").unwrap(), ESRPVersion::new(0, 1));
    }

    #[test]
    fn test_parse_invalid_versions() {
        assert!(matches!(ESRPVersion::parse(""), Err(VersionError::Empty)));
        assert!(matches!(
            ESRPVersion::parse("1"),
            Err(VersionError::InvalidFormat(_))
        ));
        assert!(matches!(
            ESRPVersion::parse("1.0.0"),
            Err(VersionError::InvalidFormat(_))
        ));
        assert!(matches!(
            ESRPVersion::parse("abc"),
            Err(VersionError::InvalidFormat(_))
        ));
        assert!(matches!(
            ESRPVersion::parse("a.0"),
            Err(VersionError::InvalidMajor(_))
        ));
        assert!(matches!(
            ESRPVersion::parse("1.b"),
            Err(VersionError::InvalidMinor(_))
        ));
        assert!(matches!(
            ESRPVersion::parse("-1.0"),
            Err(VersionError::InvalidMajor(_))
        ));
    }

    #[test]
    fn test_compatibility() {
        let v1_0 = ESRPVersion::new(1, 0);
        let v1_5 = ESRPVersion::new(1, 5);
        let v2_0 = ESRPVersion::new(2, 0);

        // Same major = compatible
        assert!(v1_0.is_compatible_with(&v1_5));
        assert!(v1_5.is_compatible_with(&v1_0));

        // Different major = incompatible
        assert!(!v1_0.is_compatible_with(&v2_0));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_display() {
        assert_eq!(ESRPVersion::new(1, 0).to_string(), "1.0");
        assert_eq!(ESRPVersion::new(2, 5).to_string(), "2.5");
    }

    #[test]
    fn test_from_str() {
        let version: ESRPVersion = "1.0".parse().unwrap();
        assert_eq!(version, ESRPVersion::new(1, 0));
    }

    #[test]
    fn test_current_version() {
        let current = ESRPVersion::current();
        assert_eq!(current.major, ESRP_MAJOR_VERSION);
        assert_eq!(current.minor, ESRP_MINOR_VERSION);
    }

    #[test]
    fn test_require_compatible() {
        let v1_0 = ESRPVersion::new(1, 0);
        let v1_5 = ESRPVersion::new(1, 5);
        let v2_0 = ESRPVersion::new(2, 0);

        assert!(v1_0.require_compatible(&v1_5).is_ok());
        assert!(v1_0.require_compatible(&v2_0).is_err());
    }

    #[test]
    fn test_is_compatible_str() {
        assert!(ESRPVersion::is_compatible_str("1.0").unwrap());
        assert!(ESRPVersion::is_compatible_str("1.5").unwrap());
        assert!(!ESRPVersion::is_compatible_str("2.0").unwrap());
    }

    #[test]
    fn test_default() {
        let default_version = ESRPVersion::default();
        assert_eq!(default_version, ESRPVersion::current());
    }

    #[test]
    fn test_is_current() {
        let current = ESRPVersion::current();
        assert!(current.is_current());

        let other = ESRPVersion::new(2, 0);
        assert!(!other.is_current());
    }
}
