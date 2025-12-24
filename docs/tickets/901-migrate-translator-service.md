# Ticket 901: Migrate Translator Service

**Phase:** 9 - Service Migration
**Priority:** Medium
**Dependencies:** Phase 8 completion
**Blocked By:** Python bindings complete

## Summary

Migrate the translator service to use ESRP request/response format.

## Tasks

1. Replace custom request type with ESRPRequest
2. Update endpoint to `/v1/execute`
3. Extract params from ESRP format
4. Return ESRP response format

## Implementation Overview

**Before:**
```rust
#[derive(Deserialize)]
struct TranslateRequest {
    text: String,
    source_lang: String,
    target_lang: String,
}
```

**After:**
```rust
async fn execute(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, StatusCode> {
    let text = request.inputs.first()
        .ok_or(StatusCode::BAD_REQUEST)?
        .data.clone();

    let source_lang = request.params["source_lang"].as_str()...
    let target_lang = request.params["target_lang"].as_str()...

    // Forward to backend...

    Ok(ESRPResponseJson(ESRPResponse { ... }))
}
```

## Acceptance Criteria

- [ ] Service accepts ESRP requests
- [ ] Service returns ESRP responses
- [ ] Integration test with Python client works
