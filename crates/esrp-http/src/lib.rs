//! # ESRP HTTP Transport
//!
//! HTTP transport layer for the Erasmus Service Request Protocol.
//!
//! This crate provides:
//! - Axum extractors for parsing ESRP requests
//! - Response helpers for mapping ESRP status to HTTP status codes
//! - Reqwest-based client for making ESRP requests
//!
//! ## Server Example
//!
//! ```ignore
//! use axum::{routing::post, Router};
//! use esrp_http::{ESRPRequestExtractor, ESRPResponseJson};
//! use esrp_core::{ESRPResponse, Status, Timing};
//! use chrono::Utc;
//!
//! async fn handle_request(
//!     ESRPRequestExtractor(request): ESRPRequestExtractor,
//! ) -> ESRPResponseJson {
//!     // Process the request...
//!     let response = ESRPResponse {
//!         esrp_version: "1.0".to_string(),
//!         request_id: request.request_id,
//!         status: Status::Succeeded,
//!         // ...
//!     };
//!     ESRPResponseJson(response)
//! }
//!
//! let app = Router::new().route("/v1/execute", post(handle_request));
//! ```
//!
//! ## Client Example
//!
//! ```ignore
//! use esrp_http::ESRPClient;
//! use esrp_core::ESRPRequest;
//!
//! let client = ESRPClient::new("http://localhost:8080");
//! let request = ESRPRequest { /* ... */ };
//! let response = client.execute(request).await?;
//! ```

mod client;
mod error;
mod extractors;
mod response;

pub use client::ESRPClient;
pub use error::{ESRPHttpError, ErrorResponse};
pub use extractors::ESRPRequestExtractor;
pub use response::{success_response, ESRPResponseJson};
