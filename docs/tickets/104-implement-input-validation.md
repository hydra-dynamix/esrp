# Ticket 104: Implement Input Validation

**Phase:** 1 - Core Protocol Types
**Priority:** High
**Dependencies:** [102 - Implement Core Types](102-implement-core-types.md), [103 - Implement Version Validation](103-implement-version-validation.md)
**Blocked By:** Tickets 102, 103

## Summary

Implement request and response validation in `esrp-core/src/validation.rs`. This ensures ESRP messages conform to the protocol specification before processing.

## Context

Validation is critical for protocol conformance. Invalid requests should be rejected early with clear error messages. Validation covers:
- Version compatibility
- Required field presence
- Field value constraints
- Semantic validity

## Tasks

1. Define `ValidationError` enum
2. Implement `validate_request()` function
3. Implement `validate_response()` function
4. Implement helper validation functions
5. Write comprehensive tests

## Implementation Details

### validation.rs

```rust
//! ESRP Validation

use crate::types::{ESRPRequest, ESRPResponse, Encoding, Status};
use crate::version::ESRPVersion;
use thiserror::Error;

/// Errors that can occur during validation
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Empty inputs list - at least one input is required")]
    EmptyInputs,

    #[error("Invalid encoding '{0}' for input '{1}'")]
    InvalidEncoding(String, String),

    #[error("Empty content type for input '{0}'")]
    EmptyContentType(String),

    #[error("Empty data for input '{0}'")]
    EmptyData(String),

    #[error("Empty input name at index {0}")]
    EmptyInputName(usize),

    #[error("Invalid workspace URI: {0}")]
    InvalidWorkspaceUri(String),

    #[error("Version mismatch: got '{got}', expected compatible with '{expected}'")]
    VersionMismatch { got: String, expected: String },

    #[error("Invalid version format: {0}")]
    InvalidVersionFormat(String),

    #[error("Empty service name in target")]
    EmptyServiceName,

    #[error("Empty operation name in target")]
    EmptyOperationName,

    #[error("Empty system name in caller")]
    EmptySystemName,

    #[error("Missing error details for failed response")]
    MissingError,

    #[error("Missing job details for accepted response")]
    MissingJob,

    #[error("Invalid artifact URI '{0}': {1}")]
    InvalidArtifactUri(String, String),

    #[error("Invalid SHA256 hash '{0}': must be 64 hex characters")]
    InvalidSha256(String),

    #[error("Artifact size cannot be zero")]
    ZeroArtifactSize,
}

/// Validate an ESRP request
///
/// # Errors
///
/// Returns `ValidationError` if the request is invalid.
///
/// # Example
///
/// ```
/// use esrp_core::{validate_request, ESRPRequest};
///
/// let request: ESRPRequest = serde_json::from_str(json)?;
/// validate_request(&request)?;
/// ```
pub fn validate_request(request: &ESRPRequest) -> Result<(), ValidationError> {
    // Validate version
    validate_version(&request.esrp_version)?;

    // Validate caller
    validate_caller(request)?;

    // Validate target
    validate_target(request)?;

    // Validate inputs
    validate_inputs(request)?;

    Ok(())
}

/// Validate an ESRP response
///
/// # Errors
///
/// Returns `ValidationError` if the response is invalid.
pub fn validate_response(response: &ESRPResponse) -> Result<(), ValidationError> {
    // Validate version
    validate_version(&response.esrp_version)?;

    // Validate status-specific requirements
    match response.status {
        Status::Failed => {
            if response.error.is_none() {
                return Err(ValidationError::MissingError);
            }
        }
        Status::Accepted => {
            if response.job.is_none() {
                return Err(ValidationError::MissingJob);
            }
        }
        Status::Succeeded => {
            // No additional requirements
        }
    }

    // Validate artifacts
    for artifact in &response.artifacts {
        validate_artifact_uri(&artifact.uri)?;
        validate_sha256(&artifact.sha256)?;
        if artifact.size_bytes == 0 {
            return Err(ValidationError::ZeroArtifactSize);
        }
    }

    Ok(())
}

/// Validate version string
fn validate_version(version: &str) -> Result<(), ValidationError> {
    let parsed = ESRPVersion::parse(version)
        .map_err(|e| ValidationError::InvalidVersionFormat(e.to_string()))?;

    let current = ESRPVersion::current();
    if !parsed.is_compatible_with(&current) {
        return Err(ValidationError::VersionMismatch {
            got: version.to_string(),
            expected: current.to_string(),
        });
    }

    Ok(())
}

/// Validate caller
fn validate_caller(request: &ESRPRequest) -> Result<(), ValidationError> {
    if request.caller.system.is_empty() {
        return Err(ValidationError::EmptySystemName);
    }
    Ok(())
}

/// Validate target
fn validate_target(request: &ESRPRequest) -> Result<(), ValidationError> {
    if request.target.service.is_empty() {
        return Err(ValidationError::EmptyServiceName);
    }
    if request.target.operation.is_empty() {
        return Err(ValidationError::EmptyOperationName);
    }
    Ok(())
}

/// Validate inputs
fn validate_inputs(request: &ESRPRequest) -> Result<(), ValidationError> {
    if request.inputs.is_empty() {
        return Err(ValidationError::EmptyInputs);
    }

    for (i, input) in request.inputs.iter().enumerate() {
        if input.name.is_empty() {
            return Err(ValidationError::EmptyInputName(i));
        }

        if input.content_type.is_empty() {
            return Err(ValidationError::EmptyContentType(input.name.clone()));
        }

        // Data can be empty for certain encodings (e.g., path to empty file)
        // but we validate workspace URIs if encoding is path
        if input.encoding == Encoding::Path {
            validate_workspace_uri_if_present(&input.data, &input.name)?;
        }
    }

    Ok(())
}

/// Validate workspace URI format if present
fn validate_workspace_uri_if_present(data: &str, input_name: &str) -> Result<(), ValidationError> {
    if data.starts_with("workspace://") {
        validate_workspace_uri(data)?;
    } else if data.is_empty() {
        return Err(ValidationError::EmptyData(input_name.to_string()));
    }
    Ok(())
}

/// Validate workspace URI format
fn validate_workspace_uri(uri: &str) -> Result<(), ValidationError> {
    if !uri.starts_with("workspace://") {
        return Err(ValidationError::InvalidWorkspaceUri(
            "URI must start with 'workspace://'".to_string(),
        ));
    }

    let rest = &uri[12..]; // Remove "workspace://"
    if rest.is_empty() {
        return Err(ValidationError::InvalidWorkspaceUri(
            "Missing namespace and path".to_string(),
        ));
    }

    // Check for path traversal
    if rest.contains("..") {
        return Err(ValidationError::InvalidWorkspaceUri(
            "Path traversal (..) not allowed".to_string(),
        ));
    }

    // Check for absolute path
    if rest.starts_with('/') {
        return Err(ValidationError::InvalidWorkspaceUri(
            "Path must be relative (no leading /)".to_string(),
        ));
    }

    Ok(())
}

/// Validate artifact URI
fn validate_artifact_uri(uri: &str) -> Result<(), ValidationError> {
    if uri.starts_with("workspace://") {
        validate_workspace_uri(uri).map_err(|e| {
            ValidationError::InvalidArtifactUri(uri.to_string(), e.to_string())
        })
    } else {
        // Allow other URI schemes (http, file, etc.)
        Ok(())
    }
}

/// Validate SHA256 hash string
fn validate_sha256(hash: &str) -> Result<(), ValidationError> {
    if hash.len() != 64 {
        return Err(ValidationError::InvalidSha256(hash.to_string()));
    }

    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidSha256(hash.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn minimal_request() -> ESRPRequest {
        ESRPRequest {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            idempotency_key: None,
            timestamp: Utc::now(),
            scope_id: None,
            causation_id: None,
            payload_hash: None,
            caller: Caller {
                system: "test".to_string(),
                agent_id: None,
                run_id: None,
            },
            target: Target {
                service: "tts".to_string(),
                operation: "synthesize".to_string(),
                variant: None,
            },
            mode: Mode::default(),
            context: None,
            inputs: vec![Input {
                name: "text".to_string(),
                content_type: "text/plain".to_string(),
                data: "Hello".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::json!({}),
            }],
            params: serde_json::json!({}),
            params_schema_ref: None,
        }
    }

    #[test]
    fn test_valid_request() {
        let request = minimal_request();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_empty_inputs() {
        let mut request = minimal_request();
        request.inputs = vec![];
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyInputs)
        ));
    }

    #[test]
    fn test_empty_system_name() {
        let mut request = minimal_request();
        request.caller.system = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptySystemName)
        ));
    }

    #[test]
    fn test_empty_service_name() {
        let mut request = minimal_request();
        request.target.service = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyServiceName)
        ));
    }

    #[test]
    fn test_invalid_version() {
        let mut request = minimal_request();
        request.esrp_version = "2.0".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_workspace_uri_traversal() {
        assert!(matches!(
            validate_workspace_uri("workspace://temp/../etc/passwd"),
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_valid_sha256() {
        let hash = "a".repeat(64);
        assert!(validate_sha256(&hash).is_ok());
    }

    #[test]
    fn test_invalid_sha256_length() {
        assert!(matches!(
            validate_sha256("abc"),
            Err(ValidationError::InvalidSha256(_))
        ));
    }

    #[test]
    fn test_invalid_sha256_chars() {
        let hash = "z".repeat(64); // 'z' is not hex
        assert!(matches!(
            validate_sha256(&hash),
            Err(ValidationError::InvalidSha256(_))
        ));
    }
}
```

## Acceptance Criteria

- [ ] Valid requests pass validation
- [ ] Empty inputs list returns `EmptyInputs` error
- [ ] Empty system name returns `EmptySystemName` error
- [ ] Empty service/operation returns appropriate error
- [ ] Incompatible version returns `VersionMismatch` error
- [ ] Path traversal (`..`) in workspace URIs is rejected
- [ ] Invalid SHA256 hashes are rejected
- [ ] Failed responses without error field are rejected
- [ ] Accepted responses without job field are rejected
- [ ] All tests pass

## Verification

```bash
cargo test --package esrp-core validation
```

## Notes

- Validation is strict but provides helpful error messages
- Workspace URI validation is basic here - full validation is in `esrp-workspace`
- SHA256 validation ensures 64 lowercase hex characters
- Consider adding validation for content-type MIME format in future

## Reference

See `docs/ESRP-SPEC.md` sections:
- "Field Semantics" for field requirements
- "Workspace Semantics" for URI validation rules
- "Error Model" for response validation
