# Ticket 102: Implement Core Types

**Phase:** 1 - Core Protocol Types
**Priority:** Critical (Blocking)
**Dependencies:** [101 - Create esrp-core Crate](101-create-esrp-core-crate.md)
**Blocked By:** Ticket 101

## Summary

Implement all ESRP type definitions in `esrp-core/src/types.rs`. These types represent the complete ESRP request/response structures as defined in the specification.

## Context

The ESRP type system is the foundation of protocol conformance. All types must:
- Derive `Debug, Clone, Serialize, Deserialize, PartialEq`
- Use appropriate serde attributes for JSON representation
- Handle optional fields correctly
- Match the specification exactly

## Tasks

1. Implement `ESRPRequest` struct
2. Implement `ESRPResponse` struct
3. Implement all supporting structs (Caller, Target, Mode, Context, etc.)
4. Implement all enums (Status, ErrorCode, RetentionPolicy, etc.)
5. Implement Job types

## Implementation Details

### types.rs

```rust
//! ESRP Core Types

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
```

## Acceptance Criteria

- [ ] All types compile
- [ ] All types derive `Debug, Clone, Serialize, Deserialize, PartialEq`
- [ ] Serde attributes match spec (camelCase vs snake_case)
- [ ] Optional fields use `Option<T>` with `skip_serializing_if`
- [ ] Default values work correctly
- [ ] Can serialize/deserialize to JSON

## Verification

```bash
cargo build --package esrp-core
cargo test --package esrp-core
```

Create a simple test to verify serialization:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = ESRPRequest {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            // ... fill in required fields
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ESRPRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, parsed);
    }
}
```

## Notes

- Use `serde(rename_all = "snake_case")` for enums that need snake_case in JSON
- Use `serde(rename_all = "SCREAMING_SNAKE_CASE")` for error codes
- The `params` field is `serde_json::Value` to allow arbitrary JSON
- `duration_ms` is `f64` to support sub-millisecond precision
- Consider adding builder pattern in future for `ESRPRequest`

## Reference

See `docs/ESRP-SPEC.md` sections:
- "Type System" for complete field definitions
- "Core Request Structure" for request format
- "Core Response Structure" for response format
