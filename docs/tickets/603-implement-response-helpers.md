# Ticket 603: Implement Response Helpers

**Phase:** 6 - HTTP Transport
**Priority:** High
**Dependencies:** [602 - Implement Axum Extractors](602-implement-axum-extractors.md)
**Blocked By:** Ticket 602

## Summary

Implement response helpers for converting ESRP responses to HTTP responses with correct status codes.

## Implementation Details

### response.rs

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use esrp_core::{ESRPResponse, Status, ErrorCode};

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
```

## Acceptance Criteria

- [ ] Status codes mapped correctly
- [ ] Response body is JSON
- [ ] Error responses include error details
