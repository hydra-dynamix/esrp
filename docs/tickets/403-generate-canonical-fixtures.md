# Ticket 403: Generate Canonical Fixtures

**Phase:** 4 - Test Fixtures
**Priority:** High
**Dependencies:** [402 - Create Request Fixtures](402-create-request-fixtures.md), [202 - Implement Canonical JSON](202-implement-canonical-json.md)
**Blocked By:** Tickets 402, 202

## Summary

Generate canonical JSON and SHA256 hash files for each request fixture. These serve as the golden reference for conformance testing.

## Context

For each request fixture, we generate:
- Canonical JSON file (sorted keys, no whitespace)
- SHA256 hash file (64-character hex string)

These are the authoritative outputs that all implementations must match byte-for-byte.

## Tasks

1. Create generation script
2. Generate canonical JSON for each request
3. Generate SHA256 hashes
4. Verify idempotency (re-running produces same output)
5. Document the generation process

## Implementation Details

### tools/generate-fixtures.sh

```bash
#!/bin/bash
# Generate canonical JSON and hashes for all request fixtures

set -e

FIXTURES_DIR="fixtures/v1"
REQUESTS_DIR="$FIXTURES_DIR/requests"
CANONICAL_DIR="$FIXTURES_DIR/canonical"

# Build the CLI first
cargo build --release --bin esrp

ESRP="./target/release/esrp"

echo "Generating canonical fixtures..."

for request in "$REQUESTS_DIR"/*.json; do
    if [ -f "$request" ]; then
        base=$(basename "$request" .json)
        echo "Processing $base..."

        # Generate canonical JSON
        $ESRP canonicalize "$request" > "$CANONICAL_DIR/${base}.json"

        # Generate SHA256 hash
        $ESRP hash "$request" > "$CANONICAL_DIR/${base}.sha256"

        echo "  Created: ${base}.json, ${base}.sha256"
    fi
done

echo "Done! Generated canonical fixtures for $(ls -1 "$REQUESTS_DIR"/*.json | wc -l) requests."

# Verify idempotency
echo ""
echo "Verifying idempotency..."

for request in "$REQUESTS_DIR"/*.json; do
    if [ -f "$request" ]; then
        base=$(basename "$request" .json)

        # Generate again and compare
        hash1=$(cat "$CANONICAL_DIR/${base}.sha256")
        hash2=$($ESRP hash "$request")

        if [ "$hash1" != "$hash2" ]; then
            echo "ERROR: Hash mismatch for $base"
            exit 1
        fi
    fi
done

echo "All fixtures verified!"
```

### Expected Outputs

For `simple_tts.json`, the canonical output should look like:

**fixtures/v1/canonical/simple_tts.json:**
```json
{"caller":{"system":"erasmus"},"esrp_version":"1.0","inputs":[{"content_type":"text/plain","data":"Hello, world!","encoding":"utf-8","metadata":{},"name":"text"}],"params":{"voice":"en-US-Standard-A"},"request_id":"550e8400-e29b-41d4-a716-446655440000","target":{"operation":"synthesize","service":"tts"},"timestamp":"2025-01-01T00:00:00Z"}
```

**fixtures/v1/canonical/simple_tts.sha256:**
```
[64-character hex hash - generated]
```

### Payload Hash Fixtures

Also generate payload hashes (for idempotency testing):

**fixtures/v1/canonical/simple_tts.payload.sha256:**
```
[64-character hex hash of {target, inputs, params}]
```

Add to the generation script:

```bash
# Generate payload hash
$ESRP payload-hash "$request" > "$CANONICAL_DIR/${base}.payload.sha256"
```

### Verification Test

Create a Rust test that verifies fixtures:

```rust
// tests/conformance/fixture_test.rs

#[test]
fn test_canonical_matches_golden() {
    let fixtures_dir = Path::new("fixtures/v1/requests");

    for entry in fs::read_dir(fixtures_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some("json".as_ref()) {
            continue;
        }

        let filename = path.file_stem().unwrap().to_str().unwrap();

        // Read request
        let request_json = fs::read_to_string(&path).unwrap();
        let request: ESRPRequest = serde_json::from_str(&request_json).unwrap();

        // Generate canonical
        let canonical = to_canonical_json(&request).unwrap();

        // Read golden
        let golden_path = format!("fixtures/v1/canonical/{}.json", filename);
        let golden = fs::read_to_string(&golden_path).unwrap();

        // Compare byte-for-byte
        assert_eq!(
            canonical,
            golden.as_bytes(),
            "Canonical mismatch for {}",
            filename
        );
    }
}

#[test]
fn test_hash_matches_golden() {
    let fixtures_dir = Path::new("fixtures/v1/requests");

    for entry in fs::read_dir(fixtures_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some("json".as_ref()) {
            continue;
        }

        let filename = path.file_stem().unwrap().to_str().unwrap();

        // Read request
        let request_json = fs::read_to_string(&path).unwrap();
        let request: ESRPRequest = serde_json::from_str(&request_json).unwrap();

        // Generate hash
        let hash = hash_canonical(&request).unwrap();

        // Read golden
        let golden_path = format!("fixtures/v1/canonical/{}.sha256", filename);
        let golden = fs::read_to_string(&golden_path).unwrap().trim().to_string();

        // Compare
        assert_eq!(hash, golden, "Hash mismatch for {}", filename);
    }
}
```

## Acceptance Criteria

- [ ] Generation script created and executable
- [ ] Canonical JSON generated for all request fixtures
- [ ] SHA256 hashes generated for all fixtures
- [ ] Payload hashes generated for all fixtures
- [ ] Re-running script produces identical output
- [ ] No whitespace in canonical JSON files
- [ ] Hashes are 64 lowercase hex characters
- [ ] Verification tests pass

## Verification

```bash
# Generate fixtures
./tools/generate-fixtures.sh

# Verify they exist
ls -la fixtures/v1/canonical/

# Verify hash format
cat fixtures/v1/canonical/simple_tts.sha256 | wc -c
# Should be 64 or 65 (with newline)

# Re-run and verify same output
./tools/generate-fixtures.sh
git status  # Should show no changes
```

## Notes

- Script must be idempotent (same input = same output)
- Use the Rust implementation as the authoritative generator
- Store hashes without trailing newline (or with consistent handling)
- Consider git pre-commit hook to verify fixtures are up to date
