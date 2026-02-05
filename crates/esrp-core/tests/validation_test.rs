//! Validation edge case tests for esrp-core

use chrono::Utc;
use esrp_core::*;
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

mod request_validation {
    use super::*;

    #[test]
    fn test_valid_minimal_request() {
        let request = minimal_request();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_valid_full_request() {
        let request = ESRPRequest {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            idempotency_key: Some("key-123".to_string()),
            timestamp: Utc::now(),
            scope_id: Some(Uuid::new_v4()),
            causation_id: Some(Uuid::new_v4()),
            payload_hash: Some("a".repeat(64)),
            caller: Caller {
                system: "erasmus".to_string(),
                agent_id: Some("agent-1".to_string()),
                run_id: Some("run-1".to_string()),
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
                parent_span_id: Some(Uuid::new_v4()),
                tags: serde_json::json!({"env": "prod"}),
            }),
            inputs: vec![
                Input {
                    name: "text".to_string(),
                    content_type: "text/plain".to_string(),
                    data: "Hello".to_string(),
                    encoding: Encoding::Utf8,
                    metadata: serde_json::json!({}),
                },
                Input {
                    name: "audio_ref".to_string(),
                    content_type: "audio/wav".to_string(),
                    data: "workspace://temp/audio.wav".to_string(),
                    encoding: Encoding::Path,
                    metadata: serde_json::json!({"sample_rate": 44100}),
                },
            ],
            params: serde_json::json!({"voice": "en-US-Standard-A"}),
            params_schema_ref: Some("https://example.com/schema.json".to_string()),
        };

        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_multiple_inputs() {
        let mut request = minimal_request();
        request.inputs = vec![
            Input {
                name: "input1".to_string(),
                content_type: "text/plain".to_string(),
                data: "data1".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::Value::Null,
            },
            Input {
                name: "input2".to_string(),
                content_type: "application/json".to_string(),
                data: "{\"key\": \"value\"}".to_string(),
                encoding: Encoding::Utf8,
                metadata: serde_json::Value::Null,
            },
            Input {
                name: "input3".to_string(),
                content_type: "application/octet-stream".to_string(),
                data: "SGVsbG8=".to_string(),
                encoding: Encoding::Base64,
                metadata: serde_json::Value::Null,
            },
        ];

        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_compatible_version() {
        let mut request = minimal_request();
        request.esrp_version = "1.5".to_string(); // Same major version
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_incompatible_version() {
        let mut request = minimal_request();
        request.esrp_version = "2.0".to_string();
        let result = validate_request(&request);
        assert!(matches!(
            result,
            Err(ValidationError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_invalid_version_format() {
        let mut request = minimal_request();
        request.esrp_version = "invalid".to_string();
        let result = validate_request(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidVersionFormat(_))
        ));
    }

    #[test]
    fn test_empty_inputs_error() {
        let mut request = minimal_request();
        request.inputs = vec![];
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyInputs)));
    }

    #[test]
    fn test_empty_input_name_error() {
        let mut request = minimal_request();
        request.inputs[0].name = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyInputName(0))));
    }

    #[test]
    fn test_empty_input_name_at_index() {
        let mut request = minimal_request();
        request.inputs.push(Input {
            name: "".to_string(),
            content_type: "text/plain".to_string(),
            data: "data".to_string(),
            encoding: Encoding::Utf8,
            metadata: serde_json::Value::Null,
        });
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyInputName(1))));
    }

    #[test]
    fn test_empty_content_type_error() {
        let mut request = minimal_request();
        request.inputs[0].content_type = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyContentType(_))));
    }

    #[test]
    fn test_empty_system_name_error() {
        let mut request = minimal_request();
        request.caller.system = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptySystemName)));
    }

    #[test]
    fn test_empty_service_name_error() {
        let mut request = minimal_request();
        request.target.service = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyServiceName)));
    }

    #[test]
    fn test_empty_operation_name_error() {
        let mut request = minimal_request();
        request.target.operation = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyOperationName)));
    }
}

mod response_validation {
    use super::*;

    #[test]
    fn test_valid_succeeded_response() {
        let response = minimal_response();
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_valid_failed_response_with_error() {
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
    fn test_failed_response_missing_error() {
        let mut response = minimal_response();
        response.status = Status::Failed;
        response.error = None;
        let result = validate_response(&response);
        assert!(matches!(result, Err(ValidationError::MissingError)));
    }

    #[test]
    fn test_valid_accepted_response_with_job() {
        let mut response = minimal_response();
        response.status = Status::Accepted;
        response.job = Some(Job {
            job_id: Uuid::new_v4(),
            state: JobState::Queued,
        });
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_accepted_response_missing_job() {
        let mut response = minimal_response();
        response.status = Status::Accepted;
        response.job = None;
        let result = validate_response(&response);
        assert!(matches!(result, Err(ValidationError::MissingJob)));
    }

    #[test]
    fn test_response_with_valid_artifact() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 1024,
            retention: RetentionPolicy::Run,
        }];
        assert!(validate_response(&response).is_ok());
    }

    #[test]
    fn test_response_with_invalid_sha256_length() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "abc".to_string(), // Too short
            size_bytes: 1024,
            retention: RetentionPolicy::Run,
        }];
        let result = validate_response(&response);
        assert!(matches!(result, Err(ValidationError::InvalidSha256(_))));
    }

    #[test]
    fn test_response_with_invalid_sha256_chars() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "z".repeat(64), // Invalid hex chars
            size_bytes: 1024,
            retention: RetentionPolicy::Run,
        }];
        let result = validate_response(&response);
        assert!(matches!(result, Err(ValidationError::InvalidSha256(_))));
    }

    #[test]
    fn test_response_with_zero_size_artifact() {
        let mut response = minimal_response();
        response.artifacts = vec![Artifact {
            artifact_id: Uuid::new_v4(),
            kind: ArtifactKind::File,
            uri: "workspace://artifacts/output.wav".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 0,
            retention: RetentionPolicy::Run,
        }];
        let result = validate_response(&response);
        assert!(matches!(result, Err(ValidationError::ZeroArtifactSize)));
    }

    #[test]
    fn test_response_with_multiple_artifacts() {
        let mut response = minimal_response();
        response.artifacts = vec![
            Artifact {
                artifact_id: Uuid::new_v4(),
                kind: ArtifactKind::File,
                uri: "workspace://artifacts/output1.wav".to_string(),
                sha256: "a".repeat(64),
                size_bytes: 1024,
                retention: RetentionPolicy::Run,
            },
            Artifact {
                artifact_id: Uuid::new_v4(),
                kind: ArtifactKind::Blob,
                uri: "https://storage.example.com/blob/123".to_string(),
                sha256: "b".repeat(64),
                size_bytes: 2048,
                retention: RetentionPolicy::Pinned,
            },
        ];
        assert!(validate_response(&response).is_ok());
    }
}

mod workspace_uri_validation {
    use super::*;

    #[test]
    fn test_valid_workspace_uri_in_path_encoding() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "workspace://temp/input.txt".to_string();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_valid_regular_path_in_path_encoding() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "/tmp/input.txt".to_string();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_empty_data_in_path_encoding() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "".to_string();
        let result = validate_request(&request);
        assert!(matches!(result, Err(ValidationError::EmptyData(_))));
    }

    #[test]
    fn test_path_traversal_rejected() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "workspace://temp/../etc/passwd".to_string();
        let result = validate_request(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_absolute_path_in_workspace_uri_rejected() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "workspace:///etc/passwd".to_string();
        let result = validate_request(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }

    #[test]
    fn test_empty_workspace_path_rejected() {
        let mut request = minimal_request();
        request.inputs[0].encoding = Encoding::Path;
        request.inputs[0].data = "workspace://".to_string();
        let result = validate_request(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidWorkspaceUri(_))
        ));
    }
}

mod version_validation {
    use super::*;

    #[test]
    fn test_version_parsing_valid() {
        assert!(ESRPVersion::parse("1.0").is_ok());
        assert!(ESRPVersion::parse("1.5").is_ok());
        assert!(ESRPVersion::parse("2.0").is_ok());
        assert!(ESRPVersion::parse("0.1").is_ok());
        assert!(ESRPVersion::parse("255.255").is_ok());
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(ESRPVersion::parse("").is_err());
        assert!(ESRPVersion::parse("1").is_err());
        assert!(ESRPVersion::parse("1.0.0").is_err());
        assert!(ESRPVersion::parse("abc").is_err());
        assert!(ESRPVersion::parse("a.0").is_err());
        assert!(ESRPVersion::parse("1.b").is_err());
        assert!(ESRPVersion::parse("-1.0").is_err());
        assert!(ESRPVersion::parse("256.0").is_err()); // Overflow u8
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = ESRPVersion::new(1, 0);
        let v1_5 = ESRPVersion::new(1, 5);
        let v2_0 = ESRPVersion::new(2, 0);

        // Same major = compatible
        assert!(v1_0.is_compatible_with(&v1_5));
        assert!(v1_5.is_compatible_with(&v1_0));

        // Different major = incompatible
        assert!(!v1_0.is_compatible_with(&v2_0));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_version_display() {
        let version = ESRPVersion::new(1, 5);
        assert_eq!(version.to_string(), "1.5");
    }

    #[test]
    fn test_version_from_str() {
        let version: ESRPVersion = "1.5".parse().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 5);
    }

    #[test]
    fn test_version_constants() {
        assert_eq!(ESRP_VERSION, "1.0");
        assert_eq!(ESRP_MAJOR_VERSION, 1);
        assert_eq!(ESRP_MINOR_VERSION, 0);
    }
}
