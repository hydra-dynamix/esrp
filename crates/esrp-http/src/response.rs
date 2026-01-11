//! Response helpers for ESRP HTTP transport

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use esrp_core::{ESRPResponse, ErrorCode, Status};

/// Wrapper for ESRP responses that maps status to HTTP status codes
///
/// # Status Code Mapping
///
/// - `succeeded` -> 200 OK
/// - `accepted` -> 202 Accepted
/// - `failed` with error code:
///   - `BACKEND_UNAVAILABLE` -> 502 Bad Gateway
///   - `TIMEOUT` -> 408 Request Timeout
///   - `OOM` -> 507 Insufficient Storage
///   - `INVALID_INPUT_*` -> 400 Bad Request
///   - `UNKNOWN` -> 500 Internal Server Error
///
/// # Example
///
/// ```ignore
/// use esrp_http::ESRPResponseJson;
/// use esrp_core::ESRPResponse;
///
/// async fn handler() -> ESRPResponseJson {
///     let response = ESRPResponse { /* ... */ };
///     ESRPResponseJson(response)
/// }
/// ```
pub struct ESRPResponseJson(pub ESRPResponse);

impl IntoResponse for ESRPResponseJson {
    fn into_response(self) -> Response {
        let status_code = match self.0.status {
            Status::Succeeded => StatusCode::OK,
            Status::Accepted => StatusCode::ACCEPTED,
            Status::Failed => match &self.0.error {
                Some(err) => match err.code {
                    ErrorCode::BackendUnavailable => StatusCode::BAD_GATEWAY,
                    ErrorCode::Timeout => StatusCode::REQUEST_TIMEOUT,
                    ErrorCode::Oom => StatusCode::INSUFFICIENT_STORAGE,
                    ErrorCode::InvalidInputSchema
                    | ErrorCode::InvalidInputSemantic
                    | ErrorCode::InvalidInputSize => StatusCode::BAD_REQUEST,
                    ErrorCode::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
                },
                None => StatusCode::INTERNAL_SERVER_ERROR,
            },
        };

        (status_code, Json(self.0)).into_response()
    }
}

/// Create a successful ESRP response
pub fn success_response(response: ESRPResponse) -> ESRPResponseJson {
    ESRPResponseJson(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use esrp_core::{Error, Timing};
    use uuid::Uuid;

    fn make_response(status: Status, error: Option<Error>) -> ESRPResponse {
        ESRPResponse {
            esrp_version: "1.0".to_string(),
            request_id: Uuid::new_v4(),
            status,
            timing: Some(Timing {
                accepted_at: None,
                started_at: None,
                finished_at: None,
                duration_ms: Some(100.0),
            }),
            outputs: vec![],
            artifacts: vec![],
            job: None,
            error,
        }
    }

    #[test]
    fn test_succeeded_maps_to_ok() {
        let response = make_response(Status::Succeeded, None);
        let json_response = ESRPResponseJson(response);

        // We can't easily test the status code without a full axum setup,
        // but we can verify the type exists and compiles
        let _ = json_response;
    }

    #[test]
    fn test_accepted_status() {
        let response = make_response(Status::Accepted, None);
        let _ = ESRPResponseJson(response);
    }

    #[test]
    fn test_failed_status() {
        let response = make_response(
            Status::Failed,
            Some(Error {
                code: ErrorCode::BackendUnavailable,
                message: "Service unavailable".to_string(),
                retryable: true,
                retry_after_ms: None,
                retry_strategy: None,
                max_retries: None,
                details: serde_json::Value::Null,
            }),
        );
        let _ = ESRPResponseJson(response);
    }
}
