# Ticket 602: Implement Axum Extractors

**Phase:** 6 - HTTP Transport
**Priority:** High
**Dependencies:** [601 - Create esrp-http Crate](601-create-esrp-http-crate.md)
**Blocked By:** Ticket 601

## Summary

Implement Axum extractors for parsing and validating ESRP requests from HTTP.

## Implementation Details

### extractors.rs

```rust
use axum::{async_trait, extract::FromRequest, http::Request, Json};
use esrp_core::{validate_request, ESRPRequest, ValidationError};

pub struct ESRPRequestExtractor(pub ESRPRequest);

#[async_trait]
impl<S, B> FromRequest<S, B> for ESRPRequestExtractor
where
    B: Send + 'static,
    S: Send + Sync,
    Json<ESRPRequest>: FromRequest<S, B>,
{
    type Rejection = ESRPHttpError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(esrp_request) = Json::<ESRPRequest>::from_request(req, state)
            .await
            .map_err(|e| ESRPHttpError::ParseError(e.to_string()))?;

        validate_request(&esrp_request)?;

        Ok(ESRPRequestExtractor(esrp_request))
    }
}
```

## Acceptance Criteria

- [ ] Valid requests extract successfully
- [ ] Invalid JSON returns 400
- [ ] Invalid ESRP schema returns 400 with details
- [ ] Version mismatch returns appropriate error
