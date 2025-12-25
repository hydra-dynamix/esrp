# Ticket 402: Create Request Fixtures

**Phase:** 4 - Test Fixtures
**Priority:** High
**Dependencies:** [401 - Create Fixture Structure](401-create-fixture-structure.md), [102 - Implement Core Types](102-implement-core-types.md)
**Blocked By:** Tickets 401, 102

## Summary

Create golden request fixture files that serve as conformance test inputs. These cover various ESRP request scenarios.

## Context

Request fixtures should cover:
- Minimal valid request (required fields only)
- Full request (all optional fields)
- Different service types
- Async mode requests
- Requests with causation chains

## Tasks

1. Create `simple_tts.json` - minimal TTS request
2. Create `batch_translation.json` - multiple inputs
3. Create `image_generation.json` - with optional fields
4. Create `async_video.json` - async mode request
5. Validate all fixtures parse correctly

## Implementation Details

### fixtures/v1/requests/simple_tts.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-01-01T00:00:00Z",
  "caller": {
    "system": "erasmus"
  },
  "target": {
    "service": "tts",
    "operation": "synthesize"
  },
  "inputs": [
    {
      "name": "text",
      "content_type": "text/plain",
      "data": "Hello, world!",
      "encoding": "utf-8",
      "metadata": {}
    }
  ],
  "params": {
    "voice": "en-US-Standard-A"
  }
}
```

### fixtures/v1/requests/batch_translation.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440001",
  "timestamp": "2025-01-01T00:00:00Z",
  "caller": {
    "system": "erasmus",
    "agent_id": "translator-agent"
  },
  "target": {
    "service": "translator",
    "operation": "translate"
  },
  "mode": {
    "type": "sync",
    "timeout_ms": 30000
  },
  "inputs": [
    {
      "name": "text_1",
      "content_type": "text/plain",
      "data": "Hello",
      "encoding": "utf-8",
      "metadata": {"index": 0}
    },
    {
      "name": "text_2",
      "content_type": "text/plain",
      "data": "World",
      "encoding": "utf-8",
      "metadata": {"index": 1}
    }
  ],
  "params": {
    "source_lang": "en",
    "target_lang": "es"
  }
}
```

### fixtures/v1/requests/image_generation.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440002",
  "idempotency_key": "user-request-12345",
  "timestamp": "2025-01-01T00:00:00Z",
  "scope_id": "660e8400-e29b-41d4-a716-446655440000",
  "causation_id": "550e8400-e29b-41d4-a716-446655440001",
  "caller": {
    "system": "erasmus",
    "agent_id": "art-agent",
    "run_id": "run-abc123"
  },
  "target": {
    "service": "art",
    "operation": "generate",
    "variant": "fast"
  },
  "mode": {
    "type": "sync",
    "timeout_ms": 120000
  },
  "context": {
    "trace_id": "770e8400-e29b-41d4-a716-446655440000",
    "span_id": "880e8400-e29b-41d4-a716-446655440000",
    "parent_span_id": "880e8400-e29b-41d4-a716-446655440001",
    "tags": {
      "env": "production",
      "user_id": "user-123"
    }
  },
  "inputs": [
    {
      "name": "prompt",
      "content_type": "text/plain",
      "data": "A serene mountain landscape at sunset",
      "encoding": "utf-8",
      "metadata": {}
    }
  ],
  "params": {
    "width": 1024,
    "height": 768,
    "style": "photorealistic",
    "seed": "42"
  },
  "params_schema_ref": "https://example.com/schemas/art-generate.json"
}
```

### fixtures/v1/requests/async_video.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440003",
  "timestamp": "2025-01-01T00:00:00Z",
  "caller": {
    "system": "erasmus"
  },
  "target": {
    "service": "video",
    "operation": "generate"
  },
  "mode": {
    "type": "async",
    "timeout_ms": 600000
  },
  "inputs": [
    {
      "name": "script",
      "content_type": "text/plain",
      "data": "A cat playing piano",
      "encoding": "utf-8",
      "metadata": {}
    }
  ],
  "params": {
    "duration_seconds": 10,
    "resolution": "1080p"
  }
}
```

### Validation Script

Create a script to validate fixtures:

```bash
#!/bin/bash
# tools/validate-fixtures.sh

for file in fixtures/v1/requests/*.json; do
  echo "Validating $file..."
  cargo run --bin esrp -- validate "$file"
done
```

## Acceptance Criteria

- [ ] `simple_tts.json` - minimal valid request
- [ ] `batch_translation.json` - multiple inputs
- [ ] `image_generation.json` - all optional fields
- [ ] `async_video.json` - async mode
- [ ] All fixtures parse with `ESRPRequest`
- [ ] All fixtures pass validation
- [ ] No floats in params (use strings or integers)
- [ ] Fixed UUIDs for reproducibility

## Verification

```bash
# Parse with esrp-core
cargo test --package esrp-core parse_fixtures

# Or using CLI (once implemented)
esrp validate fixtures/v1/requests/simple_tts.json
```

## Notes

- Use fixed UUIDs for reproducibility
- Use fixed timestamp (`2025-01-01T00:00:00Z`)
- Avoid floats in params (use strings like `"42"` for seed)
- Include variety of optional field combinations
- Keep JSON formatted for readability (canonical version is separate)
