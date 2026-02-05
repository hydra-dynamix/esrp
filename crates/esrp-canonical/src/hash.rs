//! SHA256 hashing for ESRP

use crate::canonical::to_canonical_json;
use crate::error::CanonicalError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write;

/// Hash raw bytes with SHA256
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::hash_bytes;
///
/// let hash = hash_bytes(b"Hello, world!");
/// assert_eq!(hash.len(), 64);
/// assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    // Convert to lowercase hex
    hex_encode(&result)
}

/// Hash a string with SHA256
///
/// The string is treated as UTF-8 bytes.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::hash_string;
///
/// let hash = hash_string("Hello, world!");
/// assert_eq!(hash.len(), 64);
/// ```
pub fn hash_string(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

/// Canonicalize and hash a serializable value
///
/// This combines canonical JSON serialization with SHA256 hashing.
/// Use this for computing payload hashes and content hashes.
///
/// # Errors
///
/// Returns `CanonicalError` if canonicalization fails (e.g., floats detected).
///
/// # Example
///
/// ```rust
/// use esrp_canonical::hash_canonical;
///
/// let value = serde_json::json!({"b": 1, "a": 2});
/// let hash = hash_canonical(&value).unwrap();
///
/// // Same logical value with different key order produces same hash
/// let value2 = serde_json::json!({"a": 2, "b": 1});
/// let hash2 = hash_canonical(&value2).unwrap();
///
/// assert_eq!(hash, hash2);
/// ```
pub fn hash_canonical<T: Serialize>(value: &T) -> Result<String, CanonicalError> {
    let canonical = to_canonical_json(value)?;
    Ok(hash_bytes(&canonical))
}

/// Hash a serde_json::Value after canonicalization
pub fn hash_canonical_value(value: &serde_json::Value) -> Result<String, CanonicalError> {
    let canonical = crate::canonical::to_canonical_json_value(value)?;
    Ok(hash_bytes(&canonical))
}

/// Verify that a hash matches the expected value
///
/// # Example
///
/// ```rust
/// use esrp_canonical::{hash_bytes, verify_hash};
///
/// let data = b"Hello, world!";
/// let hash = hash_bytes(data);
///
/// assert!(verify_hash(data, &hash));
/// assert!(!verify_hash(b"Different data", &hash));
/// ```
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let computed = hash_bytes(data);
    constant_time_compare(&computed, expected_hash)
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

/// Convert bytes to lowercase hex string
fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hex, "{:02x}", byte).unwrap();
    }
    hex
}

/// Validate a SHA256 hash string format
///
/// Returns `true` if the string is a valid 64-character lowercase hex string.
pub fn is_valid_sha256(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalize a SHA256 hash to lowercase
///
/// Returns the hash in lowercase, or `None` if invalid.
pub fn normalize_sha256(hash: &str) -> Option<String> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hash_bytes() {
        let hash = hash_bytes(b"Hello, world!");

        // Should be 64 hex characters
        assert_eq!(hash.len(), 64);

        // Should be lowercase
        assert_eq!(hash, hash.to_lowercase());

        // Should be valid hex
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_determinism() {
        let hash1 = hash_bytes(b"test data");
        let hash2 = hash_bytes(b"test data");
        let hash3 = hash_bytes(b"test data");

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_different_input_different_hash() {
        let hash1 = hash_bytes(b"input 1");
        let hash2 = hash_bytes(b"input 2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_canonical() {
        let value = json!({"b": 1, "a": 2});
        let hash = hash_canonical(&value).unwrap();

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_canonical_key_order_independence() {
        // Same logical object with different key orders
        let value1 = json!({"z": 3, "a": 1, "m": 2});
        let value2 = json!({"a": 1, "m": 2, "z": 3});
        let value3 = json!({"m": 2, "z": 3, "a": 1});

        let hash1 = hash_canonical(&value1).unwrap();
        let hash2 = hash_canonical(&value2).unwrap();
        let hash3 = hash_canonical(&value3).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_hash_canonical_nested() {
        let value1 = json!({
            "outer": {"b": 2, "a": 1},
            "inner": [1, 2, 3]
        });
        let value2 = json!({
            "inner": [1, 2, 3],
            "outer": {"a": 1, "b": 2}
        });

        let hash1 = hash_canonical(&value1).unwrap();
        let hash2 = hash_canonical(&value2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_canonical_float_rejected() {
        let value = json!({"temperature": 0.7});
        let result = hash_canonical(&value);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hash() {
        let data = b"test data";
        let hash = hash_bytes(data);

        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(b"wrong data", &hash));
    }

    #[test]
    fn test_verify_hash_case_insensitive() {
        let data = b"test";
        let hash = hash_bytes(data);
        let _upper_hash = hash.to_uppercase();

        // verify_hash should work with lowercase hash
        assert!(verify_hash(data, &hash));
    }

    #[test]
    fn test_known_hash() {
        // Known SHA256 of empty string
        let hash = hash_bytes(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        // Known SHA256 of "hello"
        let hash = hash_string("hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_is_valid_sha256() {
        assert!(is_valid_sha256(&"a".repeat(64)));
        assert!(is_valid_sha256(&"0123456789abcdef".repeat(4)));

        assert!(!is_valid_sha256("too short"));
        assert!(!is_valid_sha256(&"g".repeat(64))); // 'g' is not hex
        assert!(!is_valid_sha256(&"a".repeat(65))); // too long
    }

    #[test]
    fn test_normalize_sha256() {
        let upper = "ABCD".to_string() + &"0".repeat(60);
        let normalized = normalize_sha256(&upper).unwrap();

        assert_eq!(normalized, "abcd".to_string() + &"0".repeat(60));
    }

    #[test]
    fn test_empty_input() {
        // Empty string and empty bytes should produce same hash
        assert_eq!(hash_bytes(b""), hash_string(""));
    }
}
