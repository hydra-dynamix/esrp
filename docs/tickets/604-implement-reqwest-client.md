# Ticket 604: Implement Reqwest Client

**Phase:** 6 - HTTP Transport
**Priority:** High
**Dependencies:** [603 - Implement Response Helpers](603-implement-response-helpers.md)
**Blocked By:** Ticket 603

## Summary

Implement a Reqwest-based HTTP client for making ESRP requests.

## Implementation Details

### client.rs

```rust
use esrp_core::{ESRPRequest, ESRPResponse};
use reqwest::Client;
use std::time::Duration;

pub struct ESRPClient {
    client: Client,
    base_url: String,
}

impl ESRPClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .unwrap(),
            base_url: base_url.into(),
        }
    }

    pub async fn execute(&self, request: ESRPRequest) -> Result<ESRPResponse, ClientError> {
        let url = format!("{}/v1/execute", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let esrp_response: ESRPResponse = response.json().await?;
        Ok(esrp_response)
    }
}
```

## Acceptance Criteria

- [ ] Client sends valid ESRP requests
- [ ] Client parses responses
- [ ] Client handles errors gracefully
- [ ] Timeout support
