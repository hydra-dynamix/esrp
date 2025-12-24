# Ticket 605: Write HTTP Integration Tests

**Phase:** 6 - HTTP Transport
**Priority:** High
**Dependencies:** [604 - Implement Reqwest Client](604-implement-reqwest-client.md)
**Blocked By:** Ticket 604

## Summary

Write integration tests for the HTTP transport layer using a mock Axum server.

## Implementation Details

```rust
#[tokio::test]
async fn test_client_server_round_trip() {
    // Start mock server
    let app = Router::new()
        .route("/v1/execute", post(echo_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Create client
    let client = ESRPClient::new(format!("http://{}", addr));

    // Send request
    let request = create_test_request();
    let response = client.execute(request).await.unwrap();

    assert_eq!(response.status, Status::Succeeded);
}
```

## Acceptance Criteria

- [ ] Mock server works
- [ ] Client/server round-trip succeeds
- [ ] Error responses parse correctly
- [ ] Tests use fixtures
