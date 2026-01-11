//! ESRP Core Types
//!
//! This module contains all type definitions for the Erasmus Service Request Protocol.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ESRP Request structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ESRPRequest {
    pub esrp_version: String,
    pub request_id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,

    pub timestamp: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,

    pub caller: Caller,
    pub target: Target,

    #[serde(default)]
    pub mode: Mode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,

    pub inputs: Vec<Input>,

    #[serde(default)]
    pub params: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_schema_ref: Option<String>,
}

/// ESRP Response structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ESRPResponse {
    pub esrp_version: String,
    pub request_id: Uuid,
    pub status: Status,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Output>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<Artifact>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<Job>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

/// Caller information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Caller {
    pub system: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

/// Target service and operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Target {
    pub service: String,
    pub operation: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mode {
    #[serde(rename = "type")]
    pub mode_type: ModeType,

    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            mode_type: ModeType::Sync,
            timeout_ms: default_timeout(),
        }
    }
}

fn default_timeout() -> u64 {
    600_000 // 10 minutes
}

/// Mode type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModeType {
    #[default]
    Sync,
    Async,
}

/// Tracing context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub trace_id: Uuid,
    pub span_id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<Uuid>,

    #[serde(default)]
    pub tags: serde_json::Value,
}

/// Input data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Input {
    pub name: String,
    pub content_type: String,
    pub data: String,
    pub encoding: Encoding,

    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Output data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Output {
    pub name: String,
    pub content_type: String,
    pub data: String,
    pub encoding: Encoding,

    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Data encoding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    #[default]
    #[serde(rename = "utf-8")]
    Utf8,
    Base64,
    Path,
}

/// Artifact reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    pub artifact_id: Uuid,
    pub kind: ArtifactKind,
    pub uri: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub retention: RetentionPolicy,
}

/// Artifact kind
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    File,
    Blob,
}

/// Retention policy for artifacts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RetentionPolicy {
    Ephemeral,
    #[default]
    Run,
    Pinned,
}

/// Timing information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Succeeded,
    Failed,
    Accepted,
}

/// Job reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub job_id: Uuid,
    pub state: JobState,
}

/// Job state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Queued,
    Started,
    Succeeded,
    Failed,
    Cancelled,
}

/// Job event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobEvent {
    pub event_type: JobEventType,
    pub job_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Job event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobEventType {
    JobQueued,
    JobStarted,
    JobProgress,
    ArtifactCreated,
    JobCompleted,
    JobFailed,
    JobCancelled,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,

    #[serde(default)]
    pub retryable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_strategy: Option<RetryStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,

    #[serde(default)]
    pub details: serde_json::Value,
}

/// Error codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    BackendUnavailable,
    Timeout,
    Oom,
    InvalidInputSchema,
    InvalidInputSemantic,
    InvalidInputSize,
    Unknown,
}

/// Retry strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Immediate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization_roundtrip() {
        let request = ESRPRequest {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            idempotency_key: None,
            timestamp: Utc::now(),
            scope_id: None,
            causation_id: None,
            payload_hash: None,
            caller: Caller {
                system: "test-system".to_string(),
                agent_id: None,
                run_id: None,
            },
            target: Target {
                service: "translator".to_string(),
                operation: "translate".to_string(),
                variant: None,
            },
            mode: Mode::default(),
            context: None,
            inputs: vec![Input {
                name: "text".to_string(),
                content_type: "text/plain".to_string(),
                data: "Hello, world!".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::Value::Null,
            }],
            params: serde_json::json!({"source_lang": "en", "target_lang": "es"}),
            params_schema_ref: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ESRPRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, parsed);
    }

    #[test]
    fn test_response_serialization_roundtrip() {
        let response = ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Succeeded,
            timing: Some(Timing {
                accepted_at: Some(Utc::now()),
                started_at: Some(Utc::now()),
                finished_at: Some(Utc::now()),
                duration_ms: Some(123.45),
            }),
            outputs: vec![Output {
                name: "translation".to_string(),
                content_type: "text/plain".to_string(),
                data: "Hola, mundo!".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::Value::Null,
            }],
            artifacts: vec![],
            job: None,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ESRPResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }

    #[test]
    fn test_mode_default() {
        let mode = Mode::default();
        assert_eq!(mode.mode_type, ModeType::Sync);
        assert_eq!(mode.timeout_ms, 600_000);
    }

    #[test]
    fn test_encoding_serialization() {
        assert_eq!(serde_json::to_string(&Encoding::Utf8).unwrap(), "\"utf-8\"");
        assert_eq!(
            serde_json::to_string(&Encoding::Base64).unwrap(),
            "\"base64\""
        );
        assert_eq!(serde_json::to_string(&Encoding::Path).unwrap(), "\"path\"");
    }

    #[test]
    fn test_error_code_serialization() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::BackendUnavailable).unwrap(),
            "\"BACKEND_UNAVAILABLE\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::InvalidInputSchema).unwrap(),
            "\"INVALID_INPUT_SCHEMA\""
        );
    }

    #[test]
    fn test_status_serialization() {
        assert_eq!(
            serde_json::to_string(&Status::Succeeded).unwrap(),
            "\"succeeded\""
        );
        assert_eq!(
            serde_json::to_string(&Status::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&Status::Accepted).unwrap(),
            "\"accepted\""
        );
    }
}
