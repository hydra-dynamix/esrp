# Ticket 1002: Integrate with Services

**Phase:** 10 - Event Logger
**Priority:** Medium
**Dependencies:** [1001 - Create Event Logger](1001-create-event-logger.md)
**Blocked By:** Ticket 1001

## Summary

Add logging middleware to all services.

## Tasks

1. Create logging middleware for Axum
2. Configure log path via environment
3. Enable/disable via config

## Implementation Details

```rust
pub async fn logging_middleware(
    State(log): State<Arc<EventLog>>,
    ESRPRequestExtractor(request): ESRPRequestExtractor,
    next: Next,
) -> Response {
    log.log_request(&request)?;

    let response = next.run(request).await;

    // Extract ESRP response and log
    log.log_response(&request, &esrp_response)?;

    response
}
```

## Acceptance Criteria

- [ ] All service calls logged
- [ ] Can reconstruct causation chains
- [ ] No performance impact when disabled
