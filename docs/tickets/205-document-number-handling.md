# Ticket 205: Document Number Handling

**Phase:** 2 - Canonical Representation
**Priority:** Medium
**Dependencies:** [202 - Implement Canonical JSON](202-implement-canonical-json.md)
**Blocked By:** Ticket 202

## Summary

Create comprehensive documentation for number handling in ESRP canonical JSON. This includes the README for `esrp-canonical` and inline documentation explaining why floats are prohibited.

## Context

Float handling is a critical source of cross-platform incompatibility. Different languages and platforms serialize floats differently:

- JavaScript: `0.1 + 0.2` may produce `0.30000000000000004`
- Python: Different precision handling
- Rust: Consistent but different from JS/Python

ESRP solves this by prohibiting floats in canonical regions and requiring string representation.

## Tasks

1. Create `crates/esrp-canonical/README.md`
2. Add comprehensive rustdoc comments
3. Create examples directory with code samples
4. Document common pitfalls

## Implementation Details

### README.md

Create `crates/esrp-canonical/README.md`:

```markdown
# esrp-canonical

Deterministic JSON serialization and SHA256 hashing for the Erasmus Service Request Protocol (ESRP).

## Purpose

This crate provides byte-identical JSON serialization across all platforms. This is critical for:

- **Payload hashing** - Same request = same hash, regardless of platform
- **Idempotency** - Deduplicate requests based on content
- **Artifact verification** - Verify file integrity via SHA256
- **Event log integrity** - Stable references across systems

## Canonical JSON Rules

ESRP canonical JSON follows these **NORMATIVE** rules:

| Rule | Description |
|------|-------------|
| Object keys | Sorted lexicographically by UTF-8 bytes |
| Arrays | Preserve insertion order (do not sort) |
| Numbers | Integers only, no leading zeros |
| **Floats** | **PROHIBITED** - use strings |
| Whitespace | None (compact only) |
| Encoding | UTF-8 only |
| Booleans | Lowercase `true`/`false` |
| Null | Lowercase `null` |

## Float Prohibition

### Why Floats Are Banned

Floating-point numbers produce different serializations across platforms:

```javascript
// JavaScript
JSON.stringify(0.1 + 0.2)  // "0.30000000000000004"

// Python
import json
json.dumps(0.1 + 0.2)  // "0.30000000000000004" (usually)

// Rust
serde_json::to_string(&(0.1 + 0.2))  // May differ
```

This inconsistency means the same logical value can produce different hashes, breaking idempotency.

### The Solution: Use Strings

Instead of numeric floats, use string representations:

```json
// WRONG - Non-deterministic across platforms
{"temperature": 0.7}

// CORRECT - Deterministic everywhere
{"temperature": "0.7"}
```

### What Happens If You Use Floats

The `to_canonical_json()` function will return an error:

```rust
use esrp_canonical::to_canonical_json;

let value = serde_json::json!({"temp": 0.7});
let result = to_canonical_json(&value);
// Result: Err(CanonicalError::FloatNotAllowed)
```

### Integers Are Safe

Integer numbers serialize consistently across platforms:

```rust
let value = serde_json::json!({"count": 42});
let result = to_canonical_json(&value);
// Result: Ok(b'{"count":42}')
```

## Usage

### Basic Canonicalization

```rust
use esrp_canonical::to_canonical_json_string;

let value = serde_json::json!({
    "z_key": 3,
    "a_key": 1,
    "m_key": 2
});

let canonical = to_canonical_json_string(&value)?;
// Result: {"a_key":1,"m_key":2,"z_key":3}
// Keys are sorted!
```

### Hashing

```rust
use esrp_canonical::{hash_canonical, hash_bytes};

// Hash any serializable value
let value = serde_json::json!({"key": "value"});
let hash = hash_canonical(&value)?;
// Result: 64-character lowercase hex string

// Hash raw bytes
let hash = hash_bytes(b"Hello, world!");
```

### Payload Hash

```rust
use esrp_canonical::derive_payload_hash;
use esrp_core::{Target, Input, Encoding};

let target = Target {
    service: "tts".to_string(),
    operation: "synthesize".to_string(),
    variant: None,
};

let inputs = vec![Input {
    name: "text".to_string(),
    content_type: "text/plain".to_string(),
    data: "Hello".to_string(),
    encoding: Encoding::Utf8,
    metadata: serde_json::json!({}),
}];

let params = serde_json::json!({"voice": "en-US"});

let hash = derive_payload_hash(&target, &inputs, &params)?;
```

## Cross-Platform Compatibility

This crate is the **reference implementation**. All language bindings (Python, TypeScript) must produce byte-identical output.

### Conformance Testing

```bash
# Generate test fixtures
cargo run --bin esrp -- canonicalize fixtures/v1/requests/simple_tts.json

# Compare with other implementations
python -c "import esrp; print(esrp.canonicalize(...))"
```

The fixtures in `fixtures/v1/canonical/` contain expected outputs. All implementations must match exactly.

## Common Mistakes

### Mistake 1: Using Floats for Configuration

```json
// WRONG
{"temperature": 0.7, "top_p": 0.9}

// CORRECT
{"temperature": "0.7", "top_p": "0.9"}
```

### Mistake 2: Assuming Key Order Doesn't Matter

```rust
// These produce DIFFERENT non-canonical JSON:
let a = json!({"z": 1, "a": 2});
let b = json!({"a": 2, "z": 1});

// But SAME canonical JSON:
assert_eq!(
    to_canonical_json(&a)?,
    to_canonical_json(&b)?
);
```

### Mistake 3: Using Pretty-Printed JSON

```rust
// WRONG - has whitespace
serde_json::to_string_pretty(&value)?

// CORRECT - no whitespace
to_canonical_json(&value)?
```

### Mistake 4: Not Validating Before Hashing

```rust
// WRONG - might contain floats
let hash = hash_bytes(serde_json::to_vec(&value)?);

// CORRECT - validates and rejects floats
let hash = hash_canonical(&value)?;
```

## Performance

Canonicalization adds minimal overhead:

- **Typical request**: < 1ms
- **Large params (100KB)**: < 10ms
- **SHA256 hashing**: Negligible (hardware-accelerated)

## See Also

- [ESRP Specification](../../docs/ESRP-SPEC.md) - Full protocol spec
- [Quick Reference](../../docs/ESRP-QUICK-REFERENCE.md) - Developer cheat sheet
- [RFC 8259](https://tools.ietf.org/html/rfc8259) - JSON specification
- [RFC 8785](https://tools.ietf.org/html/rfc8785) - JSON Canonicalization Scheme (JCS)
```

### Inline Documentation Updates

Ensure all public functions have comprehensive rustdoc:

```rust
/// Serialize a value to canonical JSON bytes
///
/// # Canonical JSON Rules
///
/// 1. Object keys sorted lexicographically by UTF-8 bytes
/// 2. Arrays preserve insertion order
/// 3. No whitespace
/// 4. Floats are **PROHIBITED** (use strings like `"0.7"`)
///
/// # Why Floats Are Prohibited
///
/// Floating-point serialization varies across platforms:
/// - JavaScript: `0.1 + 0.2` → `0.30000000000000004`
/// - Python/Rust: May differ
///
/// This breaks cross-platform hash consistency. Use strings instead:
/// ```json
/// {"temperature": "0.7"}  // Correct
/// {"temperature": 0.7}    // WRONG - will error
/// ```
///
/// # Errors
///
/// Returns [`CanonicalError::FloatNotAllowed`] if any float is detected.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::to_canonical_json;
///
/// let value = serde_json::json!({"z": 1, "a": 2});
/// let bytes = to_canonical_json(&value)?;
/// assert_eq!(bytes, b"{\"a\":2,\"z\":1}");
/// ```
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError>
```

## Acceptance Criteria

- [ ] README.md created with comprehensive documentation
- [ ] Float prohibition explained with rationale
- [ ] Cross-platform examples included
- [ ] Common mistakes documented
- [ ] All public functions have rustdoc
- [ ] Examples compile and work

## Verification

```bash
# Build documentation
cargo doc --package esrp-canonical --open

# Verify examples compile
cargo test --package esrp-canonical --doc
```

## Notes

- Documentation is as important as code for protocol adoption
- Include specific platform examples (JS, Python, Rust)
- Link to relevant RFCs
- Make the "why" clear, not just the "what"
