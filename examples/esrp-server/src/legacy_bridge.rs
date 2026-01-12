//! Legacy bridge to Erasmus services
//!
//! This module translates ESRP requests to the legacy Erasmus protocol
//! and converts responses back to ESRP format.

use chrono::Utc;
use esrp_core::{
    ESRPRequest, ESRPResponse, Encoding, Error, ErrorCode, Output, Status, Timing,
};
use serde::{Deserialize, Serialize};

/// Legacy Erasmus request format
#[derive(Debug, Serialize)]
struct LegacyRequest {
    message: LegacyMessage,
    config: serde_json::Value,
}

/// Legacy Erasmus message format
#[derive(Debug, Serialize, Deserialize)]
struct LegacyMessage {
    content: String,
    content_type: String,
    #[serde(default)]
    encoding: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

/// Legacy Erasmus response format
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LegacyResponse {
    success: bool,
    message: Option<LegacyMessage>,
    error: Option<String>,
    duration_ms: Option<f64>,
    timestamp: Option<String>,
}

pub struct LegacyBridge;

impl LegacyBridge {
    /// Call the legacy translator service
    pub async fn translate(
        erasmus_url: &str,
        request: &ESRPRequest,
    ) -> Result<ESRPResponse, ESRPResponse> {
        let started_at = Utc::now();

        // Extract input text
        let input = request
            .inputs
            .iter()
            .find(|i| i.name == "text")
            .ok_or_else(|| Self::error_response(request, "Missing input: text", started_at))?;

        // Build legacy request
        let legacy_req = LegacyRequest {
            message: LegacyMessage {
                content: input.data.clone(),
                content_type: input.content_type.clone(),
                encoding: match input.encoding {
                    Encoding::Utf8 => "utf-8".to_string(),
                    Encoding::Base64 => "base64".to_string(),
                    Encoding::Path => "path".to_string(),
                },
                metadata: serde_json::json!({
                    "config": {
                        "source_lang": request.params.get("source_lang").and_then(|v| v.as_str()).unwrap_or("en"),
                        "target_lang": request.params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("es")
                    }
                }),
            },
            config: serde_json::json!({
                "source_lang": request.params.get("source_lang").and_then(|v| v.as_str()).unwrap_or("en"),
                "target_lang": request.params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("es")
            }),
        };

        // Call legacy service
        let url = format!("{}/api/v1/services/translator/process", erasmus_url);
        let response = Self::call_legacy(&url, &legacy_req, request, started_at).await?;

        // Convert response
        Self::convert_response(request, response, "translation", started_at)
    }

    /// Call the legacy TTS service
    pub async fn tts(
        erasmus_url: &str,
        request: &ESRPRequest,
    ) -> Result<ESRPResponse, ESRPResponse> {
        let started_at = Utc::now();

        // Extract input text
        let input = request
            .inputs
            .iter()
            .find(|i| i.name == "text")
            .ok_or_else(|| Self::error_response(request, "Missing input: text", started_at))?;

        // Build legacy request
        let legacy_req = LegacyRequest {
            message: LegacyMessage {
                content: input.data.clone(),
                content_type: "text/plain".to_string(),
                encoding: "utf-8".to_string(),
                metadata: serde_json::json!({
                    "config": {
                        "voice": request.params.get("voice").and_then(|v| v.as_str()).unwrap_or("default")
                    }
                }),
            },
            config: serde_json::json!({
                "voice": request.params.get("voice").and_then(|v| v.as_str()).unwrap_or("default")
            }),
        };

        // Call legacy service
        let url = format!("{}/api/v1/services/text2speech/process", erasmus_url);
        let response = Self::call_legacy(&url, &legacy_req, request, started_at).await?;

        // Convert response - TTS returns audio
        Self::convert_audio_response(request, response, "audio", started_at)
    }

    /// Call the legacy STT service
    pub async fn stt(
        erasmus_url: &str,
        request: &ESRPRequest,
    ) -> Result<ESRPResponse, ESRPResponse> {
        let started_at = Utc::now();

        // Extract audio input
        let input = request
            .inputs
            .iter()
            .find(|i| i.name == "audio")
            .ok_or_else(|| Self::error_response(request, "Missing input: audio", started_at))?;

        // Build legacy request
        let legacy_req = LegacyRequest {
            message: LegacyMessage {
                content: input.data.clone(),
                content_type: input.content_type.clone(),
                encoding: match input.encoding {
                    Encoding::Utf8 => "utf-8".to_string(),
                    Encoding::Base64 => "base64".to_string(),
                    Encoding::Path => "path".to_string(),
                },
                metadata: serde_json::Value::Null,
            },
            config: serde_json::json!({}),
        };

        // Call legacy service
        let url = format!("{}/api/v1/services/speech2text/process", erasmus_url);
        let response = Self::call_legacy(&url, &legacy_req, request, started_at).await?;

        // Convert response
        Self::convert_response(request, response, "transcription", started_at)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn call_legacy(
        url: &str,
        legacy_req: &LegacyRequest,
        request: &ESRPRequest,
        started_at: chrono::DateTime<Utc>,
    ) -> Result<LegacyResponse, ESRPResponse> {
        tracing::debug!("Calling legacy service at: {}", url);

        let client = reqwest::Client::new();
        let result = client
            .post(url)
            .json(legacy_req)
            .timeout(std::time::Duration::from_millis(
                request.mode.timeout_ms,
            ))
            .send()
            .await;

        match result {
            Ok(resp) => {
                if resp.status().is_success() {
                    resp.json::<LegacyResponse>()
                        .await
                        .map_err(|e| Self::error_response(request, &format!("Failed to parse response: {}", e), started_at))
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    Err(Self::error_response(
                        request,
                        &format!("Service returned {}: {}", status, body),
                        started_at,
                    ))
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    Err(Self::timeout_response(request, started_at))
                } else if e.is_connect() {
                    Err(Self::unavailable_response(request, &e.to_string(), started_at))
                } else {
                    Err(Self::error_response(request, &e.to_string(), started_at))
                }
            }
        }
    }

    fn convert_response(
        request: &ESRPRequest,
        legacy: LegacyResponse,
        output_name: &str,
        started_at: chrono::DateTime<Utc>,
    ) -> Result<ESRPResponse, ESRPResponse> {
        let finished_at = Utc::now();

        if !legacy.success {
            return Err(Self::error_response(
                request,
                legacy.error.as_deref().unwrap_or("Unknown error"),
                started_at,
            ));
        }

        let message = legacy
            .message
            .ok_or_else(|| Self::error_response(request, "No message in response", started_at))?;

        Ok(ESRPResponse {
            esrp_version: request.esrp_version.clone(),
            request_id: request.request_id,
            status: Status::Succeeded,
            timing: Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: legacy.duration_ms.or(Some(
                    (finished_at - started_at).num_milliseconds() as f64,
                )),
            }),
            outputs: vec![Output {
                name: output_name.to_string(),
                content_type: message.content_type,
                data: message.content,
                encoding: match message.encoding.as_str() {
                    "base64" => Encoding::Base64,
                    "path" => Encoding::Path,
                    _ => Encoding::Utf8,
                },
                metadata: message.metadata,
            }],
            artifacts: vec![],
            job: None,
            error: None,
        })
    }

    fn convert_audio_response(
        request: &ESRPRequest,
        legacy: LegacyResponse,
        output_name: &str,
        started_at: chrono::DateTime<Utc>,
    ) -> Result<ESRPResponse, ESRPResponse> {
        let finished_at = Utc::now();

        if !legacy.success {
            return Err(Self::error_response(
                request,
                legacy.error.as_deref().unwrap_or("Unknown error"),
                started_at,
            ));
        }

        let message = legacy
            .message
            .ok_or_else(|| Self::error_response(request, "No message in response", started_at))?;

        // Audio is typically returned as base64
        let encoding = if message.encoding == "path" {
            Encoding::Path
        } else {
            Encoding::Base64
        };

        Ok(ESRPResponse {
            esrp_version: request.esrp_version.clone(),
            request_id: request.request_id,
            status: Status::Succeeded,
            timing: Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: legacy.duration_ms.or(Some(
                    (finished_at - started_at).num_milliseconds() as f64,
                )),
            }),
            outputs: vec![Output {
                name: output_name.to_string(),
                content_type: message.content_type,
                data: message.content,
                encoding,
                metadata: message.metadata,
            }],
            artifacts: vec![],
            job: None,
            error: None,
        })
    }

    fn error_response(
        request: &ESRPRequest,
        message: &str,
        started_at: chrono::DateTime<Utc>,
    ) -> ESRPResponse {
        let finished_at = Utc::now();
        ESRPResponse {
            esrp_version: request.esrp_version.clone(),
            request_id: request.request_id,
            status: Status::Failed,
            timing: Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: Some((finished_at - started_at).num_milliseconds() as f64),
            }),
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: Some(Error {
                code: ErrorCode::Unknown,
                message: message.to_string(),
                retryable: false,
                retry_after_ms: None,
                retry_strategy: None,
                max_retries: None,
                details: serde_json::Value::Null,
            }),
        }
    }

    fn timeout_response(
        request: &ESRPRequest,
        started_at: chrono::DateTime<Utc>,
    ) -> ESRPResponse {
        let finished_at = Utc::now();
        ESRPResponse {
            esrp_version: request.esrp_version.clone(),
            request_id: request.request_id,
            status: Status::Failed,
            timing: Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: Some((finished_at - started_at).num_milliseconds() as f64),
            }),
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: Some(Error {
                code: ErrorCode::Timeout,
                message: "Request timed out".to_string(),
                retryable: true,
                retry_after_ms: Some(1000),
                retry_strategy: Some(esrp_core::RetryStrategy::Exponential),
                max_retries: Some(3),
                details: serde_json::Value::Null,
            }),
        }
    }

    fn unavailable_response(
        request: &ESRPRequest,
        error: &str,
        started_at: chrono::DateTime<Utc>,
    ) -> ESRPResponse {
        let finished_at = Utc::now();
        ESRPResponse {
            esrp_version: request.esrp_version.clone(),
            request_id: request.request_id,
            status: Status::Failed,
            timing: Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: Some((finished_at - started_at).num_milliseconds() as f64),
            }),
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error: Some(Error {
                code: ErrorCode::BackendUnavailable,
                message: format!("Backend service unavailable: {}", error),
                retryable: true,
                retry_after_ms: Some(5000),
                retry_strategy: Some(esrp_core::RetryStrategy::Exponential),
                max_retries: Some(5),
                details: serde_json::Value::Null,
            }),
        }
    }
}
