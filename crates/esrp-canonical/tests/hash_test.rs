//! Hash tests for esrp-canonical

use esrp_canonical::{hash_bytes, hash_canonical, hash_string, verify_hash};
use serde_json::json;

#[test]
fn test_known_empty_hash() {
    let hash = hash_bytes(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_known_hello_hash() {
    let hash = hash_string("hello");
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_canonical_hash_key_order_independence() {
    let v1 = json!({"b": 2, "a": 1});
    let v2 = json!({"a": 1, "b": 2});

    let h1 = hash_canonical(&v1).unwrap();
    let h2 = hash_canonical(&v2).unwrap();

    assert_eq!(h1, h2);
}

#[test]
fn test_verify_hash_correct() {
    let data = b"test data";
    let hash = hash_bytes(data);
    assert!(verify_hash(data, &hash));
}

#[test]
fn test_verify_hash_incorrect() {
    let data = b"test data";
    let wrong_hash = "a".repeat(64);
    assert!(!verify_hash(data, &wrong_hash));
}

#[test]
fn test_hash_format() {
    let hash = hash_bytes(b"test");

    // Should be 64 characters
    assert_eq!(hash.len(), 64);

    // Should be lowercase hex
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(hash, hash.to_lowercase());
}

#[test]
fn test_hash_determinism() {
    let data = b"determinism test";

    let hashes: Vec<_> = (0..100).map(|_| hash_bytes(data)).collect();

    let first = &hashes[0];
    for hash in &hashes[1..] {
        assert_eq!(first, hash);
    }
}

#[test]
fn test_different_data_different_hash() {
    let h1 = hash_bytes(b"data 1");
    let h2 = hash_bytes(b"data 2");

    assert_ne!(h1, h2);
}

#[test]
fn test_hash_canonical_nested_key_order() {
    let v1 = json!({
        "outer": {"b": 2, "a": 1},
        "inner": [1, 2, 3]
    });
    let v2 = json!({
        "inner": [1, 2, 3],
        "outer": {"a": 1, "b": 2}
    });

    let h1 = hash_canonical(&v1).unwrap();
    let h2 = hash_canonical(&v2).unwrap();

    assert_eq!(h1, h2);
}
