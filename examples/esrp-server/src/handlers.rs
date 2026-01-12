//! ESRP request handlers

use crate::legacy_bridge::LegacyBridge;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use esrp_core::{
    ESRPRequest, ESRPResponse, Encoding, Error, ErrorCode, Output, Status, Timing,
};
use esrp_http::{ESRPRequestExtractor, ESRPResponseJson};
use serde::Serialize;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    mode: String,
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    let mode = if std::env::var("ERASMUS_URL").is_ok() {
        "proxy"
    } else {
        "mock"
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: esrp_core::ESRPVersion::current().to_string(),
        mode: mode.to_string(),
    })
}

/// Main execute endpoint - routes to appropriate handler based on target
pub async fn execute(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, (StatusCode, Json<ESRPResponse>)> {
    let started_at = Utc::now();

    tracing::info!(
        request_id = %request.request_id,
        service = %request.target.service,
        operation = %request.target.operation,
        "Processing ESRP request"
    );

    // Route based on service
    let result = match request.target.service.as_str() {
        "translator" => handle_translate(&request).await,
        "tts" => handle_tts(&request).await,
        "stt" => handle_stt(&request).await,
        "video" => handle_video(&request).await,
        _ => {
            // Unknown service - return error
            Err(create_error_response(
                &request,
                ErrorCode::InvalidInputSemantic,
                format!("Unknown service: {}", request.target.service),
                started_at,
            ))
        }
    };

    match result {
        Ok(mut response) => {
            // Add timing info
            let finished_at = Utc::now();
            response.timing = Some(Timing {
                accepted_at: Some(started_at),
                started_at: Some(started_at),
                finished_at: Some(finished_at),
                duration_ms: Some(
                    (finished_at - started_at).num_milliseconds() as f64,
                ),
            });
            Ok(ESRPResponseJson(response))
        }
        Err(response) => Err((StatusCode::BAD_REQUEST, Json(response))),
    }
}

/// Convenience endpoint for translation
pub async fn translate(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, (StatusCode, Json<ESRPResponse>)> {
    execute(ESRPRequestExtractor(ESRPRequest {
        target: esrp_core::Target {
            service: "translator".to_string(),
            operation: "translate".to_string(),
            variant: request.target.variant,
        },
        ..request
    }))
    .await
}

/// Convenience endpoint for TTS
pub async fn text_to_speech(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, (StatusCode, Json<ESRPResponse>)> {
    execute(ESRPRequestExtractor(ESRPRequest {
        target: esrp_core::Target {
            service: "tts".to_string(),
            operation: "synthesize".to_string(),
            variant: request.target.variant,
        },
        ..request
    }))
    .await
}

/// Convenience endpoint for STT
pub async fn speech_to_text(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, (StatusCode, Json<ESRPResponse>)> {
    execute(ESRPRequestExtractor(ESRPRequest {
        target: esrp_core::Target {
            service: "stt".to_string(),
            operation: "transcribe".to_string(),
            variant: request.target.variant,
        },
        ..request
    }))
    .await
}

// ============================================================================
// Service Handlers
// ============================================================================

async fn handle_translate(request: &ESRPRequest) -> Result<ESRPResponse, ESRPResponse> {
    // Check if we should proxy to real service
    if let Ok(erasmus_url) = std::env::var("ERASMUS_URL") {
        return LegacyBridge::translate(&erasmus_url, request).await;
    }

    // Mock mode - return simulated translation
    let input_text = get_input_text(request, "text")?;
    let source_lang = request
        .params
        .get("source_lang")
        .and_then(|v| v.as_str())
        .unwrap_or("en");
    let target_lang = request
        .params
        .get("target_lang")
        .and_then(|v| v.as_str())
        .unwrap_or("es");

    tracing::debug!(
        "Mock translation: {} -> {} for text: {}",
        source_lang,
        target_lang,
        &input_text[..input_text.len().min(50)]
    );

    // Simple mock - just prefix with target language
    let translated = format!("[{}] {}", target_lang.to_uppercase(), input_text);

    Ok(ESRPResponse {
        esrp_version: request.esrp_version.clone(),
        request_id: request.request_id,
        status: Status::Succeeded,
        timing: None,
        outputs: vec![Output {
            name: "translation".to_string(),
            content_type: "text/plain".to_string(),
            data: translated,
            encoding: Encoding::Utf8,
            metadata: serde_json::json!({
                "source_lang": source_lang,
                "target_lang": target_lang
            }),
        }],
        artifacts: vec![],
        job: None,
        error: None,
    })
}

async fn handle_tts(request: &ESRPRequest) -> Result<ESRPResponse, ESRPResponse> {
    // Check if we should proxy to real service
    if let Ok(erasmus_url) = std::env::var("ERASMUS_URL") {
        return LegacyBridge::tts(&erasmus_url, request).await;
    }

    // Mock mode - return fake audio data
    let input_text = get_input_text(request, "text")?;
    let voice = request
        .params
        .get("voice")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    tracing::debug!(
        "Mock TTS: voice={} for text: {}",
        voice,
        &input_text[..input_text.len().min(50)]
    );

    // Generate fake WAV header + silence (44 bytes header + some zeros)
    let fake_wav: Vec<u8> = {
        let wav = vec![
            0x52, 0x49, 0x46, 0x46, // "RIFF"
            0x24, 0x00, 0x00, 0x00, // File size - 8 (36 bytes)
            0x57, 0x41, 0x56, 0x45, // "WAVE"
            0x66, 0x6d, 0x74, 0x20, // "fmt "
            0x10, 0x00, 0x00, 0x00, // Subchunk1 size (16)
            0x01, 0x00, // Audio format (1 = PCM)
            0x01, 0x00, // Num channels (1)
            0x44, 0xac, 0x00, 0x00, // Sample rate (44100)
            0x88, 0x58, 0x01, 0x00, // Byte rate (88200)
            0x02, 0x00, // Block align (2)
            0x10, 0x00, // Bits per sample (16)
            0x64, 0x61, 0x74, 0x61, // "data"
            0x00, 0x00, 0x00, 0x00, // Data size (0 for mock)
        ];
        wav
    };

    let audio_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &fake_wav);

    Ok(ESRPResponse {
        esrp_version: request.esrp_version.clone(),
        request_id: request.request_id,
        status: Status::Succeeded,
        timing: None,
        outputs: vec![Output {
            name: "audio".to_string(),
            content_type: "audio/wav".to_string(),
            data: audio_base64,
            encoding: Encoding::Base64,
            metadata: serde_json::json!({
                "voice": voice,
                "sample_rate": 44100,
                "channels": 1,
                "mock": true
            }),
        }],
        artifacts: vec![],
        job: None,
        error: None,
    })
}

async fn handle_stt(request: &ESRPRequest) -> Result<ESRPResponse, ESRPResponse> {
    // Check if we should proxy to real service
    if let Ok(erasmus_url) = std::env::var("ERASMUS_URL") {
        return LegacyBridge::stt(&erasmus_url, request).await;
    }

    // Mock mode - return fake transcription
    let _audio_input = request
        .inputs
        .iter()
        .find(|i| i.name == "audio")
        .ok_or_else(|| {
            create_error_response(
                request,
                ErrorCode::InvalidInputSchema,
                "Missing required input: audio".to_string(),
                Utc::now(),
            )
        })?;

    tracing::debug!("Mock STT: transcribing audio");

    Ok(ESRPResponse {
        esrp_version: request.esrp_version.clone(),
        request_id: request.request_id,
        status: Status::Succeeded,
        timing: None,
        outputs: vec![Output {
            name: "transcription".to_string(),
            content_type: "text/plain".to_string(),
            data: "[Mock transcription] This is a simulated transcription of the audio input."
                .to_string(),
            encoding: Encoding::Utf8,
            metadata: serde_json::json!({
                "mock": true,
                "language": "en"
            }),
        }],
        artifacts: vec![],
        job: None,
        error: None,
    })
}

async fn handle_video(request: &ESRPRequest) -> Result<ESRPResponse, ESRPResponse> {
    // Video generation is always async, return accepted status
    tracing::debug!("Mock video: would start async job");

    // For mock, just return accepted
    Ok(ESRPResponse {
        esrp_version: request.esrp_version.clone(),
        request_id: request.request_id,
        status: Status::Accepted,
        timing: None,
        outputs: vec![],
        artifacts: vec![],
        job: Some(esrp_core::Job {
            job_id: uuid::Uuid::new_v4(),
            state: esrp_core::JobState::Queued,
        }),
        error: None,
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_input_text(request: &ESRPRequest, name: &str) -> Result<String, ESRPResponse> {
    let input = request
        .inputs
        .iter()
        .find(|i| i.name == name)
        .ok_or_else(|| {
            create_error_response(
                request,
                ErrorCode::InvalidInputSchema,
                format!("Missing required input: {}", name),
                Utc::now(),
            )
        })?;

    // Decode based on encoding
    match input.encoding {
        Encoding::Utf8 => Ok(input.data.clone()),
        Encoding::Base64 => {
            let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &input.data)
                .map_err(|e| {
                    create_error_response(
                        request,
                        ErrorCode::InvalidInputSchema,
                        format!("Invalid base64 in input '{}': {}", name, e),
                        Utc::now(),
                    )
                })?;
            String::from_utf8(bytes).map_err(|e| {
                create_error_response(
                    request,
                    ErrorCode::InvalidInputSchema,
                    format!("Invalid UTF-8 in input '{}': {}", name, e),
                    Utc::now(),
                )
            })
        }
        Encoding::Path => {
            // Read from file path
            std::fs::read_to_string(&input.data).map_err(|e| {
                create_error_response(
                    request,
                    ErrorCode::InvalidInputSchema,
                    format!("Failed to read file '{}': {}", input.data, e),
                    Utc::now(),
                )
            })
        }
    }
}

fn create_error_response(
    request: &ESRPRequest,
    code: ErrorCode,
    message: String,
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
            code,
            message,
            retryable: false,
            retry_after_ms: None,
            retry_strategy: None,
            max_retries: None,
            details: serde_json::Value::Null,
        }),
    }
}
