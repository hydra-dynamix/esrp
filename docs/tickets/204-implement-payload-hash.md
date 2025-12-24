# Ticket 204: Implement Payload Hash Derivation

**Phase:** 2 - Canonical Representation
**Priority:** High
**Dependencies:** [202 - Implement Canonical JSON](202-implement-canonical-json.md), [203 - Implement SHA256 Hashing](203-implement-hashing.md)
**Blocked By:** Tickets 202, 203

## Summary

Implement `derive_payload_hash()` and `derive_idempotency_key()` functions in `esrp-canonical/src/payload.rs`. These functions compute the canonical hash used for request deduplication.

## Context

The payload hash is defined in the spec as:

```
payload_hash = sha256(canonical_json({
  "target": {
    "service": "...",
    "operation": "...",
    "variant": "..." | null
  },
  "inputs": [...],
  "params": {...}
}))
```

Including `target` prevents accidental deduplication when the same input is sent to different operations.

The `idempotency_key` is derived as `payload_hash` when not explicitly provided.

## Tasks

1. Implement `derive_payload_hash()` function
2. Implement `derive_idempotency_key()` alias
3. Implement `compute_request_payload_hash()` helper for full requests
4. Handle null vs missing variant correctly
5. Write comprehensive tests

## Implementation Details

### payload.rs

```rust
//! Payload hash derivation for ESRP

use crate::error::CanonicalError;
use crate::hash::hash_canonical;
use esrp_core::{ESRPRequest, Input, Target};
use serde::Serialize;
use serde_json::{json, Value};

/// Derive the payload hash from target, inputs, and params
///
/// This is the authoritative implementation of the payload hash formula:
///
/// ```text
/// payload_hash = sha256(canonical_json({
///   "target": { service, operation, variant },
///   "inputs": [...],
///   "params": {...}
/// }))
/// ```
///
/// # Arguments
///
/// * `target` - The service target (service, operation, variant)
/// * `inputs` - The input data array
/// * `params` - The parameters object
///
/// # Errors
///
/// Returns `CanonicalError` if canonicalization fails (e.g., floats in params).
///
/// # Example
///
/// ```rust
/// use esrp_canonical::derive_payload_hash;
/// use esrp_core::{Target, Input, Encoding};
///
/// let target = Target {
///     service: "tts".to_string(),
///     operation: "synthesize".to_string(),
///     variant: None,
/// };
///
/// let inputs = vec![Input {
///     name: "text".to_string(),
///     content_type: "text/plain".to_string(),
///     data: "Hello".to_string(),
///     encoding: Encoding::Utf8,
///     metadata: serde_json::json!({}),
/// }];
///
/// let params = serde_json::json!({"voice": "en-US"});
///
/// let hash = derive_payload_hash(&target, &inputs, &params)?;
/// assert_eq!(hash.len(), 64);
/// ```
pub fn derive_payload_hash(
    target: &Target,
    inputs: &[Input],
    params: &Value,
) -> Result<String, CanonicalError> {
    // Build the payload object according to spec
    let payload = build_payload_object(target, inputs, params);

    // Canonicalize and hash
    hash_canonical(&payload)
}

/// Derive the idempotency key from target, inputs, and params
///
/// This is an alias for `derive_payload_hash()` as specified in the protocol.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::{derive_payload_hash, derive_idempotency_key};
/// use esrp_core::{Target, Input, Encoding};
///
/// let target = Target { /* ... */ };
/// let inputs = vec![];
/// let params = serde_json::json!({});
///
/// // These produce identical results
/// let hash = derive_payload_hash(&target, &inputs, &params)?;
/// let key = derive_idempotency_key(&target, &inputs, &params)?;
/// assert_eq!(hash, key);
/// ```
pub fn derive_idempotency_key(
    target: &Target,
    inputs: &[Input],
    params: &Value,
) -> Result<String, CanonicalError> {
    derive_payload_hash(target, inputs, params)
}

/// Compute the payload hash for a complete ESRP request
///
/// Convenience function that extracts target, inputs, and params from the request.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::compute_request_payload_hash;
/// use esrp_core::ESRPRequest;
///
/// let request: ESRPRequest = // ...
/// let hash = compute_request_payload_hash(&request)?;
/// ```
pub fn compute_request_payload_hash(request: &ESRPRequest) -> Result<String, CanonicalError> {
    derive_payload_hash(&request.target, &request.inputs, &request.params)
}

/// Verify that a request's payload_hash matches the computed value
///
/// Returns `true` if the payload_hash field matches the computed hash,
/// or if payload_hash is None.
pub fn verify_request_payload_hash(request: &ESRPRequest) -> Result<bool, CanonicalError> {
    match &request.payload_hash {
        Some(provided_hash) => {
            let computed = compute_request_payload_hash(request)?;
            Ok(provided_hash == &computed)
        }
        None => Ok(true), // No hash provided, nothing to verify
    }
}

/// Build the payload object for hashing
fn build_payload_object(target: &Target, inputs: &[Input], params: &Value) -> Value {
    // Build target object - include variant as null if None
    let target_obj = json!({
        "service": target.service,
        "operation": target.operation,
        "variant": target.variant
    });

    // Build inputs array with only the fields needed for hashing
    let inputs_array: Vec<Value> = inputs
        .iter()
        .map(|input| {
            json!({
                "name": input.name,
                "content_type": input.content_type,
                "data": input.data,
                "encoding": input.encoding,
                "metadata": input.metadata
            })
        })
        .collect();

    // Build final payload object
    json!({
        "target": target_obj,
        "inputs": inputs_array,
        "params": params
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use esrp_core::Encoding;
    use serde_json::json;

    fn test_target() -> Target {
        Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: None,
        }
    }

    fn test_input() -> Input {
        Input {
            name: "text".to_string(),
            content_type: "text/plain".to_string(),
            data: "Hello".to_string(),
            encoding: Encoding::Utf8,
            metadata: json!({}),
        }
    }

    #[test]
    fn test_derive_payload_hash() {
        let hash = derive_payload_hash(
            &test_target(),
            &[test_input()],
            &json!({"voice": "en-US"}),
        )
        .unwrap();

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_same_inputs_same_hash() {
        let hash1 = derive_payload_hash(
            &test_target(),
            &[test_input()],
            &json!({"a": 1}),
        )
        .unwrap();

        let hash2 = derive_payload_hash(
            &test_target(),
            &[test_input()],
            &json!({"a": 1}),
        )
        .unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_params_different_hash() {
        let hash1 = derive_payload_hash(
            &test_target(),
            &[test_input()],
            &json!({"voice": "en-US"}),
        )
        .unwrap();

        let hash2 = derive_payload_hash(
            &test_target(),
            &[test_input()],
            &json!({"voice": "en-GB"}),
        )
        .unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_different_target_different_hash() {
        let target1 = Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: None,
        };

        let target2 = Target {
            service: "translator".to_string(),
            operation: "translate".to_string(),
            variant: None,
        };

        let hash1 = derive_payload_hash(&target1, &[test_input()], &json!({})).unwrap();
        let hash2 = derive_payload_hash(&target2, &[test_input()], &json!({})).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_variant_affects_hash() {
        let target1 = Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: None,
        };

        let target2 = Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: Some("fast".to_string()),
        };

        let hash1 = derive_payload_hash(&target1, &[], &json!({})).unwrap();
        let hash2 = derive_payload_hash(&target2, &[], &json!({})).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_idempotency_key_equals_payload_hash() {
        let target = test_target();
        let inputs = vec![test_input()];
        let params = json!({"key": "value"});

        let hash = derive_payload_hash(&target, &inputs, &params).unwrap();
        let key = derive_idempotency_key(&target, &inputs, &params).unwrap();

        assert_eq!(hash, key);
    }

    #[test]
    fn test_params_key_order_irrelevant() {
        let params1 = json!({"z": 3, "a": 1, "m": 2});
        let params2 = json!({"a": 1, "m": 2, "z": 3});

        let hash1 = derive_payload_hash(&test_target(), &[], &params1).unwrap();
        let hash2 = derive_payload_hash(&test_target(), &[], &params2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_float_in_params_rejected() {
        let params = json!({"temperature": 0.7});
        let result = derive_payload_hash(&test_target(), &[], &params);

        assert!(result.is_err());
    }

    #[test]
    fn test_float_as_string_accepted() {
        let params = json!({"temperature": "0.7"});
        let result = derive_payload_hash(&test_target(), &[], &params);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_inputs() {
        let hash = derive_payload_hash(&test_target(), &[], &json!({})).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_multiple_inputs() {
        let inputs = vec![
            Input {
                name: "input1".to_string(),
                content_type: "text/plain".to_string(),
                data: "data1".to_string(),
                encoding: Encoding::Utf8,
                metadata: json!({}),
            },
            Input {
                name: "input2".to_string(),
                content_type: "text/plain".to_string(),
                data: "data2".to_string(),
                encoding: Encoding::Utf8,
                metadata: json!({}),
            },
        ];

        let hash = derive_payload_hash(&test_target(), &inputs, &json!({})).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_input_order_matters() {
        let input1 = Input {
            name: "a".to_string(),
            content_type: "text/plain".to_string(),
            data: "data_a".to_string(),
            encoding: Encoding::Utf8,
            metadata: json!({}),
        };

        let input2 = Input {
            name: "b".to_string(),
            content_type: "text/plain".to_string(),
            data: "data_b".to_string(),
            encoding: Encoding::Utf8,
            metadata: json!({}),
        };

        let hash1 = derive_payload_hash(&test_target(), &[input1.clone(), input2.clone()], &json!({})).unwrap();
        let hash2 = derive_payload_hash(&test_target(), &[input2, input1], &json!({})).unwrap();

        // Arrays preserve order, so different order = different hash
        assert_ne!(hash1, hash2);
    }
}
```

## Acceptance Criteria

- [ ] Same target/inputs/params produces same hash
- [ ] Different target/inputs/params produces different hash
- [ ] Target service, operation, and variant all affect hash
- [ ] Params key order does not affect hash (canonical sorting)
- [ ] Input order affects hash (arrays preserve order)
- [ ] Floats in params cause error
- [ ] Floats as strings are accepted
- [ ] Empty inputs array works
- [ ] `derive_idempotency_key()` equals `derive_payload_hash()`
- [ ] All tests pass

## Verification

```bash
cargo test --package esrp-canonical payload
```

## Notes

- The `variant` field is included as `null` when `None` to ensure consistent hashing
- Input order matters because arrays are not sorted (per spec)
- Params order doesn't matter because objects are canonically sorted
- The payload structure matches the spec exactly
- Consider adding a `PayloadHashBuilder` for fluent API in future

## Reference

See `docs/ESRP-SPEC.md` section "Payload Hash Computation" for the authoritative definition.
