//! HTTP integration tests using mock Axum server

use axum::{routing::post, Router};
use chrono::Utc;
use esrp_core::{Caller, ESRPRequest, ESRPResponse, Encoding, Input, Mode, Status, Target, Timing};
use esrp_http::{ESRPClient, ESRPRequestExtractor, ESRPResponseJson};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

/// Echo handler that validates the request and returns a success response
async fn echo_handler(ESRPRequestExtractor(request): ESRPRequestExtractor) -> ESRPResponseJson {
    let response = ESRPResponse {
        esrp_version: request.esrp_version,
        request_id: request.request_id,
        status: Status::Succeeded,
        timing: Some(Timing {
            accepted_at: None,
            started_at: None,
            finished_at: None,
            duration_ms: Some(42.0),
        }),
        outputs: vec![],
        artifacts: vec![],
        job: None,
        error: None,
    };
    ESRPResponseJson(response)
}

/// Create a test ESRP request
fn create_test_request() -> ESRPRequest {
    ESRPRequest {
        esrp_version: "1.0".to_string(),
        request_id: Uuid::new_v4(),
        idempotency_key: None,
        timestamp: Utc::now(),
        scope_id: None,
        causation_id: None,
        payload_hash: None,
        caller: Caller {
            system: "test-client".to_string(),
            agent_id: None,
            run_id: None,
        },
        target: Target {
            service: "test-service".to_string(),
            operation: "process".to_string(),
            variant: None,
        },
        mode: Mode::default(),
        context: None,
        inputs: vec![Input {
            name: "text".to_string(),
            content_type: "text/plain".to_string(),
            data: "Hello, ESRP!".to_string(),
            encoding: Encoding::Utf8,
            metadata: serde_json::Value::Null,
        }],
        params: serde_json::json!({}),
        params_schema_ref: None,
    }
}

/// Start a test server and return its address
async fn start_test_server() -> SocketAddr {
    let app = Router::new().route("/v1/execute", post(echo_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    addr
}

#[tokio::test]
async fn test_client_server_round_trip() {
    let addr = start_test_server().await;
    let client = ESRPClient::new(format!("http://{}", addr));

    let request = create_test_request();
    let request_id = request.request_id;

    let response = client.execute(request).await.unwrap();

    assert_eq!(response.status, Status::Succeeded);
    assert_eq!(response.request_id, request_id);
    assert_eq!(response.esrp_version, "1.0");
    assert!(response.timing.is_some());
}

#[tokio::test]
async fn test_multiple_requests() {
    let addr = start_test_server().await;
    let client = ESRPClient::new(format!("http://{}", addr));

    // Send multiple requests
    for _ in 0..5 {
        let request = create_test_request();
        let response = client.execute(request).await.unwrap();
        assert_eq!(response.status, Status::Succeeded);
    }
}

#[tokio::test]
async fn test_execute_at_custom_path() {
    let app = Router::new().route("/custom/endpoint", post(echo_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let client = ESRPClient::new(format!("http://{}", addr));
    let request = create_test_request();

    let response = client.execute_at("custom/endpoint", request).await.unwrap();
    assert_eq!(response.status, Status::Succeeded);
}

#[tokio::test]
async fn test_client_with_custom_reqwest_client() {
    let addr = start_test_server().await;

    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let client = ESRPClient::with_client(custom_client, format!("http://{}", addr));

    let request = create_test_request();
    let response = client.execute(request).await.unwrap();

    assert_eq!(response.status, Status::Succeeded);
}

#[tokio::test]
async fn test_request_to_nonexistent_server_fails() {
    let client = ESRPClient::new("http://127.0.0.1:1");
    let request = create_test_request();

    let result = client.execute(request).await;
    assert!(result.is_err());
}
