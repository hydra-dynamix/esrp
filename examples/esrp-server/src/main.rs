//! ESRP Test Server
//!
//! A local test server that implements the ESRP protocol.
//! Can run in mock mode or proxy to real Erasmus services.
//!
//! Usage:
//!   # Mock mode (default)
//!   cargo run --package esrp-server
//!
//!   # Proxy to real Erasmus services
//!   ERASMUS_URL=http://localhost:8000 cargo run --package esrp-server

// Allow large error types - ESRPResponse is used as both success and error
#![allow(clippy::result_large_err)]

mod handlers;
mod legacy_bridge;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "esrp_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Check if we're in proxy mode
    let erasmus_url = std::env::var("ERASMUS_URL").ok();
    if let Some(url) = &erasmus_url {
        tracing::info!("Running in PROXY mode - forwarding to Erasmus at {}", url);
    } else {
        tracing::info!("Running in MOCK mode - returning simulated responses");
    }

    // Build router
    let app = Router::new()
        // ESRP v1 endpoints
        .route("/v1/execute", post(handlers::execute))
        .route("/v1/health", get(handlers::health))
        // Service-specific endpoints (convenience)
        .route("/v1/translate", post(handlers::translate))
        .route("/v1/tts", post(handlers::text_to_speech))
        .route("/v1/stt", post(handlers::speech_to_text))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("ESRP server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
