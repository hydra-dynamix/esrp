//! Axum extractors for ESRP requests

use crate::error::ESRPHttpError;
use async_trait::async_trait;
use axum::extract::{FromRequest, Request};
use axum::Json;
use esrp_core::{validate_request, ESRPRequest};

/// Axum extractor for validated ESRP requests
///
/// This extractor parses the request body as JSON, deserializes it to an
/// `ESRPRequest`, and validates it according to the ESRP specification.
///
/// # Example
///
/// ```ignore
/// use axum::{routing::post, Router};
/// use esrp_http::ESRPRequestExtractor;
///
/// async fn handler(ESRPRequestExtractor(request): ESRPRequestExtractor) {
///     // request is a validated ESRPRequest
/// }
///
/// let app = Router::new().route("/execute", post(handler));
/// ```
pub struct ESRPRequestExtractor(pub ESRPRequest);

#[async_trait]
impl<S> FromRequest<S> for ESRPRequestExtractor
where
    S: Send + Sync,
{
    type Rejection = ESRPHttpError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(esrp_request) = Json::<ESRPRequest>::from_request(req, state)
            .await
            .map_err(|e| ESRPHttpError::ParseError(e.to_string()))?;

        validate_request(&esrp_request)?;

        Ok(ESRPRequestExtractor(esrp_request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_type_exists() {
        // Compile-time check that the type exists
        fn _assert_extractor(_: ESRPRequestExtractor) {}
    }
}
