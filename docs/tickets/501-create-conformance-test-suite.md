# Ticket 501: Create Conformance Test Suite

**Phase:** 5 - Conformance Tests
**Priority:** High
**Dependencies:** [403 - Generate Canonical Fixtures](403-generate-canonical-fixtures.md)
**Blocked By:** Phase 4 completion

## Summary

Create the Rust conformance test suite that validates all fixtures parse correctly and produce expected canonical output and hashes.

## Context

Conformance tests ensure the Rust implementation correctly handles all fixtures. These tests are the reference for other language implementations.

## Tasks

1. Create test module structure
2. Implement fixture parsing tests
3. Implement canonical JSON comparison tests
4. Implement hash verification tests
5. Add round-trip tests

## Implementation Details

### tests/conformance/mod.rs

```rust
//! ESRP Conformance Test Suite

use esrp_canonical::{hash_canonical, to_canonical_json};
use esrp_core::{validate_request, validate_response, ESRPRequest, ESRPResponse};
use std::fs;
use std::path::Path;

const FIXTURES_DIR: &str = "fixtures/v1";

fn request_fixtures() -> Vec<(String, String)> {
    let dir = Path::new(FIXTURES_DIR).join("requests");
    fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .map(|e| {
            let path = e.path();
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).unwrap();
            (name, content)
        })
        .collect()
}

#[test]
fn test_parse_all_request_fixtures() {
    for (name, json) in request_fixtures() {
        let request: ESRPRequest = serde_json::from_str(&json)
            .expect(&format!("Failed to parse request fixture: {}", name));

        validate_request(&request)
            .expect(&format!("Failed to validate request fixture: {}", name));
    }
}

#[test]
fn test_canonical_json_matches_golden() {
    for (name, json) in request_fixtures() {
        let request: ESRPRequest = serde_json::from_str(&json).unwrap();

        let canonical = to_canonical_json(&request).unwrap();

        let golden_path = format!("{}/canonical/{}.json", FIXTURES_DIR, name);
        let golden = fs::read(&golden_path)
            .expect(&format!("Missing golden file: {}", golden_path));

        assert_eq!(
            canonical, golden,
            "Canonical JSON mismatch for {}\nGot: {}\nExpected: {}",
            name,
            String::from_utf8_lossy(&canonical),
            String::from_utf8_lossy(&golden)
        );
    }
}

#[test]
fn test_hashes_match_expected() {
    for (name, json) in request_fixtures() {
        let request: ESRPRequest = serde_json::from_str(&json).unwrap();

        let hash = hash_canonical(&request).unwrap();

        let golden_path = format!("{}/canonical/{}.sha256", FIXTURES_DIR, name);
        let expected = fs::read_to_string(&golden_path)
            .expect(&format!("Missing hash file: {}", golden_path))
            .trim()
            .to_string();

        assert_eq!(
            hash, expected,
            "Hash mismatch for {}\nGot: {}\nExpected: {}",
            name, hash, expected
        );
    }
}

#[test]
fn test_round_trip() {
    for (name, json) in request_fixtures() {
        let request: ESRPRequest = serde_json::from_str(&json).unwrap();

        // Serialize and parse again
        let serialized = serde_json::to_string(&request).unwrap();
        let parsed: ESRPRequest = serde_json::from_str(&serialized).unwrap();

        // Canonical should be identical
        let canonical1 = to_canonical_json(&request).unwrap();
        let canonical2 = to_canonical_json(&parsed).unwrap();

        assert_eq!(
            canonical1, canonical2,
            "Round-trip changed canonical JSON for {}",
            name
        );
    }
}

#[test]
fn test_parse_all_response_fixtures() {
    let dir = Path::new(FIXTURES_DIR).join("responses");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|x| x == "json").unwrap_or(false) {
            let name = path.file_stem().unwrap().to_string_lossy();
            let json = fs::read_to_string(&path).unwrap();

            let response: ESRPResponse = serde_json::from_str(&json)
                .expect(&format!("Failed to parse response fixture: {}", name));

            validate_response(&response)
                .expect(&format!("Failed to validate response fixture: {}", name));
        }
    }
}
```

## Acceptance Criteria

- [ ] All request fixtures parse
- [ ] All request fixtures validate
- [ ] Canonical JSON matches golden byte-for-byte
- [ ] Hashes match expected values
- [ ] Round-trip preserves canonical form
- [ ] All response fixtures parse and validate
- [ ] Tests run in CI

## Verification

```bash
cargo test conformance
```
