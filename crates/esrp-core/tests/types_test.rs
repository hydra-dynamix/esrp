//! Type serialization tests for esrp-core

use chrono::Utc;
use esrp_core::*;
use uuid::Uuid;

mod serialization {
    use super::*;

    #[test]
    fn test_request_round_trip() {
        let request = ESRPRequest {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            idempotency_key: Some("abc123".to_string()),
            timestamp: Utc::now(),
            scope_id: Some(Uuid::new_v4()),
            causation_id: None,
            payload_hash: Some("a".repeat(64)),
            caller: Caller {
                system: "erasmus".to_string(),
                agent_id: Some("agent-1".to_string()),
                run_id: None,
            },
            target: Target {
                service: "tts".to_string(),
                operation: "synthesize".to_string(),
                variant: Some("fast".to_string()),
            },
            mode: Mode {
                mode_type: ModeType::Async,
                timeout_ms: 30000,
            },
            context: Some(Context {
                trace_id: Uuid::new_v4(),
                span_id: Uuid::new_v4(),
                parent_span_id: None,
                tags: serde_json::json!({"env": "test"}),
            }),
            inputs: vec![Input {
                name: "text".to_string(),
                content_type: "text/plain".to_string(),
                data: "Hello, world!".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::json!({}),
            }],
            params: serde_json::json!({"voice": "en-US-Standard-A"}),
            params_schema_ref: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ESRPRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, parsed);
    }

    #[test]
    fn test_response_round_trip() {
        let response = ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Succeeded,
            timing: Some(Timing {
                accepted_at: Some(Utc::now()),
                started_at: Some(Utc::now()),
                finished_at: Some(Utc::now()),
                duration_ms: Some(1234.5),
            }),
            outputs: vec![Output {
                name: "audio".to_string(),
                content_type: "audio/wav".to_string(),
                data: "workspace://artifacts/audio.wav".to_string(),
                encoding: Encoding::Path,
                metadata: serde_json::json!({}),
            }],
            artifacts: vec![Artifact {
                artifact_id: Uuid::new_v4(),
                kind: ArtifactKind::File,
                uri: "workspace://artifacts/audio.wav".to_string(),
                sha256: "a".repeat(64),
                size_bytes: 12345,
                retention: RetentionPolicy::Run,
            }],
            job: None,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ESRPResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, parsed);
    }

    #[test]
    fn test_error_response_round_trip() {
        let response = ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Failed,
            timing: None,
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: Some(Error {
                code: ErrorCode::BackendUnavailable,
                message: "Service temporarily unavailable".to_string(),
                retryable: true,
                retry_after_ms: Some(5000),
                retry_strategy: Some(RetryStrategy::Exponential),
                max_retries: Some(3),
                details: serde_json::json!({"backend": "tts-service"}),
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ESRPResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, parsed);
    }

    #[test]
    fn test_async_response_round_trip() {
        let response = ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Accepted,
            timing: Some(Timing {
                accepted_at: Some(Utc::now()),
                started_at: None,
                finished_at: None,
                duration_ms: None,
            }),
            outputs: vec![],
            artifacts: vec![],
            job: Some(Job {
                job_id: Uuid::new_v4(),
                state: JobState::Queued,
            }),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ESRPResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, parsed);
    }
}

mod defaults {
    use super::*;

    #[test]
    fn test_mode_defaults() {
        let mode = Mode::default();
        assert_eq!(mode.mode_type, ModeType::Sync);
        assert_eq!(mode.timeout_ms, 600_000);
    }

    #[test]
    fn test_retention_policy_default() {
        let policy = RetentionPolicy::default();
        assert_eq!(policy, RetentionPolicy::Run);
    }

    #[test]
    fn test_encoding_default() {
        let encoding = Encoding::default();
        assert_eq!(encoding, Encoding::Utf8);
    }

    #[test]
    fn test_mode_type_default() {
        let mode_type = ModeType::default();
        assert_eq!(mode_type, ModeType::Sync);
    }

    #[test]
    fn test_version_default() {
        let version = ESRPVersion::default();
        assert_eq!(version, ESRPVersion::current());
    }
}

mod optional_fields {
    use super::*;

    #[test]
    fn test_optional_fields_omitted_in_json() {
        let caller = Caller {
            system: "test".to_string(),
            agent_id: None,
            run_id: None,
        };

        let json = serde_json::to_string(&caller).unwrap();

        // Optional None fields should not appear in JSON
        assert!(!json.contains("agent_id"));
        assert!(!json.contains("run_id"));
    }

    #[test]
    fn test_optional_fields_parsed_when_missing() {
        let json = r#"{"system": "test"}"#;
        let caller: Caller = serde_json::from_str(json).unwrap();

        assert_eq!(caller.system, "test");
        assert!(caller.agent_id.is_none());
        assert!(caller.run_id.is_none());
    }

    #[test]
    fn test_empty_vec_omitted_in_json() {
        let response = ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status: Status::Succeeded,
            timing: None,
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();

        // Empty vecs with skip_serializing_if should not appear
        assert!(!json.contains("outputs"));
        assert!(!json.contains("artifacts"));
    }

    #[test]
    fn test_optional_target_variant() {
        let target = Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: None,
        };

        let json = serde_json::to_string(&target).unwrap();
        assert!(!json.contains("variant"));

        let parsed: Target = serde_json::from_str(&json).unwrap();
        assert!(parsed.variant.is_none());
    }
}

mod json_parsing {
    use super::*;

    #[test]
    fn test_minimal_request_json() {
        let json = r#"{
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "erasmus"},
            "target": {"service": "tts", "operation": "synthesize"},
            "inputs": [{"name": "text", "content_type": "text/plain", "data": "Hello", "encoding": "utf-8"}],
            "params": {}
        }"#;

        let request: ESRPRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.esrp_version, "1.0");
        assert_eq!(request.caller.system, "erasmus");
        assert_eq!(request.target.service, "tts");
        assert_eq!(request.inputs.len(), 1);
        // Default mode should be applied
        assert_eq!(request.mode.mode_type, ModeType::Sync);
        assert_eq!(request.mode.timeout_ms, 600_000);
    }

    #[test]
    fn test_error_code_serialization() {
        // Verify SCREAMING_SNAKE_CASE
        let code = ErrorCode::BackendUnavailable;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"BACKEND_UNAVAILABLE\"");

        let code = ErrorCode::InvalidInputSchema;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"INVALID_INPUT_SCHEMA\"");

        let code = ErrorCode::Oom;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"OOM\"");
    }

    #[test]
    fn test_status_serialization() {
        // Verify lowercase
        let status = Status::Succeeded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"succeeded\"");

        let status = Status::Failed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"failed\"");

        let status = Status::Accepted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"accepted\"");
    }

    #[test]
    fn test_mode_type_in_json() {
        let mode = Mode {
            mode_type: ModeType::Async,
            timeout_ms: 30000,
        };

        let json = serde_json::to_string(&mode).unwrap();

        // Should use "type" not "mode_type"
        assert!(json.contains("\"type\":\"async\""));
        assert!(!json.contains("mode_type"));
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
    fn test_job_state_serialization() {
        assert_eq!(
            serde_json::to_string(&JobState::Queued).unwrap(),
            "\"queued\""
        );
        assert_eq!(
            serde_json::to_string(&JobState::Started).unwrap(),
            "\"started\""
        );
        assert_eq!(
            serde_json::to_string(&JobState::Succeeded).unwrap(),
            "\"succeeded\""
        );
        assert_eq!(
            serde_json::to_string(&JobState::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&JobState::Cancelled).unwrap(),
            "\"cancelled\""
        );
    }

    #[test]
    fn test_artifact_kind_serialization() {
        assert_eq!(
            serde_json::to_string(&ArtifactKind::File).unwrap(),
            "\"file\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactKind::Blob).unwrap(),
            "\"blob\""
        );
    }

    #[test]
    fn test_retention_policy_serialization() {
        assert_eq!(
            serde_json::to_string(&RetentionPolicy::Ephemeral).unwrap(),
            "\"ephemeral\""
        );
        assert_eq!(
            serde_json::to_string(&RetentionPolicy::Run).unwrap(),
            "\"run\""
        );
        assert_eq!(
            serde_json::to_string(&RetentionPolicy::Pinned).unwrap(),
            "\"pinned\""
        );
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_unicode_in_data() {
        let input = Input {
            name: "text".to_string(),
            content_type: "text/plain".to_string(),
            data: "Hello 世界 🌍".to_string(),
            encoding: Encoding::Utf8,
            metadata: serde_json::json!({}),
        };

        let json = serde_json::to_string(&input).unwrap();
        let parsed: Input = serde_json::from_str(&json).unwrap();

        assert_eq!(input.data, parsed.data);
    }

    #[test]
    fn test_large_params() {
        let large_value = "x".repeat(100_000);
        let params = serde_json::json!({"large": large_value});

        // Should serialize without issues
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(params, parsed);
    }

    #[test]
    fn test_zero_timeout() {
        let mode = Mode {
            mode_type: ModeType::Sync,
            timeout_ms: 0,
        };

        let json = serde_json::to_string(&mode).unwrap();
        let parsed: Mode = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.timeout_ms, 0);
    }

    #[test]
    fn test_max_timeout() {
        let mode = Mode {
            mode_type: ModeType::Sync,
            timeout_ms: u64::MAX,
        };

        let json = serde_json::to_string(&mode).unwrap();
        let parsed: Mode = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.timeout_ms, u64::MAX);
    }

    #[test]
    fn test_special_characters_in_strings() {
        let caller = Caller {
            system: "test\n\t\"\\".to_string(),
            agent_id: None,
            run_id: None,
        };

        let json = serde_json::to_string(&caller).unwrap();
        let parsed: Caller = serde_json::from_str(&json).unwrap();

        assert_eq!(caller.system, parsed.system);
    }

    #[test]
    fn test_empty_string_fields() {
        let target = Target {
            service: "".to_string(),
            operation: "".to_string(),
            variant: Some("".to_string()),
        };

        let json = serde_json::to_string(&target).unwrap();
        let parsed: Target = serde_json::from_str(&json).unwrap();

        assert_eq!(target, parsed);
    }

    #[test]
    fn test_nested_json_in_params() {
        let params = serde_json::json!({
            "nested": {
                "deeply": {
                    "nested": {
                        "value": [1, 2, 3]
                    }
                }
            }
        });

        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(params, parsed);
    }
}

mod malformed_json {
    use super::*;

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"esrp_version": "1.0"}"#;
        let result: Result<ESRPRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uuid() {
        let json = r#"{
            "esrp_version": "1.0",
            "request_id": "not-a-uuid",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "test"},
            "target": {"service": "tts", "operation": "synthesize"},
            "inputs": [],
            "params": {}
        }"#;

        let result: Result<ESRPRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_timestamp() {
        let json = r#"{
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "not-a-timestamp",
            "caller": {"system": "test"},
            "target": {"service": "tts", "operation": "synthesize"},
            "inputs": [],
            "params": {}
        }"#;

        let result: Result<ESRPRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_encoding_value() {
        let json = r#"{"name": "test", "content_type": "text/plain", "data": "hello", "encoding": "invalid"}"#;
        let result: Result<Input, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_status_value() {
        let json = r#"{"esrp_version": "1.0", "request_id": "550e8400-e29b-41d4-a716-446655440000", "status": "invalid"}"#;
        let result: Result<ESRPResponse, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_error_code_value() {
        let json = r#"{"code": "INVALID_CODE", "message": "test"}"#;
        let result: Result<Error, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_type_for_field() {
        let json = r#"{"system": 123}"#;
        let result: Result<Caller, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_array_instead_of_object() {
        let json = r#"[]"#;
        let result: Result<ESRPRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
