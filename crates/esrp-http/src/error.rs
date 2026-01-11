//! HTTP error types for ESRP

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use esrp_core::ValidationError;
use serde::Serialize;
use thiserror::Error;

/// HTTP errors for ESRP operations
#[derive(Debug, Error)]
pub enum ESRPHttpError {
    #[error("Failed to parse request: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Client error: {0}")]
    ClientError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ESRPHttpError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match &self {
            ESRPHttpError::ParseError(msg) => {
                (StatusCode::BAD_REQUEST, "PARSE_ERROR", msg.clone(), None)
            }
            ESRPHttpError::ValidationError(e) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                e.to_string(),
                Some(format!("{:?}", e)),
            ),
            ESRPHttpError::ClientError(msg) => {
                (StatusCode::BAD_REQUEST, "CLIENT_ERROR", msg.clone(), None)
            }
            ESRPHttpError::ServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                msg.clone(),
                None,
            ),
            ESRPHttpError::RequestError(e) => (
                StatusCode::BAD_GATEWAY,
                "REQUEST_ERROR",
                e.to_string(),
                None,
            ),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
            details,
        };

        (status, Json(body)).into_response()
    }
}
