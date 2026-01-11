//! ESRP Validation
//!
//! This module provides validation logic for ESRP requests and responses.
//! Validation ensures ESRP messages conform to the protocol specification before processing.

use crate::types::{ESRPRequest, ESRPResponse, Encoding, Status};
use crate::version::ESRPVersion;
use thiserror::Error;

/// Errors that can occur during validation
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field}: {reason}")]
    InvalidValue { field: String, reason: String },

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
/// ```ignore
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

    for (index, input) in request.inputs.iter().enumerate() {
        if input.name.is_empty() {
            return Err(ValidationError::EmptyInputName(index));
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
        validate_workspace_uri(uri)
            .map_err(|e| ValidationError::InvalidArtifactUri(uri.to_string(), e.to_string()))
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

    fn minimal_response() -> ESRPResponse {
        ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Succeeded,
            timing: None,
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: None,
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
    fn test_empty_operation_name() {
        let mut request = minimal_request();
        request.target.operation = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyOperationName)
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
    fn test_empty_input_name() {
        let mut request = minimal_request();
        request.inputs[0].name = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyInputName(0))
        ));
    }

    #[test]
    fn test_empty_content_type() {
        let mut request = minimal_request();
        request.inputs[0].content_type = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyContentType(_))
        ));
    }

    #[test]
    fn test_valid_response() {
        let response = minimal_response();
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_failed_response_missing_error() {
        let mut response = minimal_response();
        response.status = Status::Failed;
        assert!(matches!(
            validate_response(&response),
            Err(ValidationError::MissingError)
        ));
    }

    #[test]
    fn test_failed_response_with_error() {
        let mut response = minimal_response();
        response.status = Status::Failed;
        response.error = Some(Error {
            code: ErrorCode::Unknown,
            message: "Something went wrong".to_string(),
            retryable: false,
            retry_after_ms: None,
            retry_strategy: None,
            max_retries: None,
            details: serde_json::Value::Null,
        });
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_accepted_response_missing_job() {
        let mut response = minimal_response();
        response.status = Status::Accepted;
        assert!(matches!(
            validate_response(&response),
            Err(ValidationError::MissingJob)
        ));
    }

    #[test]
    fn test_accepted_response_with_job() {
        let mut response = minimal_response();
        response.status = Status::Accepted;
        response.job = Some(Job {
            job_id: Uuid::new_v4(),
            state: JobState::Queued,
        });
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_workspace_uri_valid() {
        assert!(validate_workspace_uri("workspace://temp/file.txt").is_ok());
        assert!(validate_workspace_uri("workspace://artifacts/output.wav").is_ok());
    }

    #[test]
    fn test_workspace_uri_traversal() {
        assert!(matches!(
            validate_workspace_uri("workspace://temp/../etc/passwd"),
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_workspace_uri_absolute_path() {
        assert!(matches!(
            validate_workspace_uri("workspace:///etc/passwd"),
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_workspace_uri_empty_path() {
        assert!(matches!(
            validate_workspace_uri("workspace://"),
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_valid_sha256() {
        let hash = "a".repeat(64);
        assert!(validate_sha256(&hash).is_ok());

        let hash = "0123456789abcdef".repeat(4);
        assert!(validate_sha256(&hash).is_ok());
    }

    #[test]
    fn test_invalid_sha256_length() {
        assert!(matches!(
            validate_sha256("abc"),
            Err(ValidationError::InvalidSha256(_))
        ));

        let too_long = "a".repeat(65);
        assert!(matches!(
            validate_sha256(&too_long),
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

    #[test]
    fn test_artifact_with_invalid_sha256() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "invalid".to_string(),
            size_bytes: 1024,
            retention: RetentionPolicy::Run,
        }];
        assert!(matches!(
            validate_response(&response),
            Err(ValidationError::InvalidSha256(_))
        ));
    }

    #[test]
    fn test_artifact_with_zero_size() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 0,
            retention: RetentionPolicy::Run,
        }];
        assert!(matches!(
            validate_response(&response),
            Err(ValidationError::ZeroArtifactSize)
        ));
    }

    #[test]
    fn test_path_encoding_empty_data() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "".to_string();
        assert!(matches!(
            validate_request(&request),
            Err(ValidationError::EmptyData(_))
        ));
    }

    #[test]
    fn test_path_encoding_valid_workspace_uri() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "workspace://temp/input.txt".to_string();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_path_encoding_regular_path() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "/tmp/input.txt".to_string();
        assert!(validate_request(&request).is_ok());
    }
}
