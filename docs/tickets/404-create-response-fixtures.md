# Ticket 404: Create Response Fixtures

**Phase:** 4 - Test Fixtures
**Priority:** Medium
**Dependencies:** [401 - Create Fixture Structure](401-create-fixture-structure.md), [102 - Implement Core Types](102-implement-core-types.md)
**Blocked By:** Tickets 401, 102

## Summary

Create golden response fixture files covering success, error, and async accepted scenarios.

## Context

Response fixtures demonstrate:
- Successful response with outputs and artifacts
- Error responses with structured error details
- Async accepted responses with job info
- Different status codes and timing information

## Tasks

1. Create success response fixtures
2. Create error response fixtures
3. Create async accepted response fixtures
4. Validate all fixtures parse correctly

## Implementation Details

### fixtures/v1/responses/simple_tts_success.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "succeeded",
  "timing": {
    "accepted_at": "2025-01-01T00:00:00Z",
    "started_at": "2025-01-01T00:00:00.100Z",
    "finished_at": "2025-01-01T00:00:01.234Z",
    "duration_ms": 1134
  },
  "outputs": [
    {
      "name": "audio",
      "content_type": "audio/wav",
      "data": "workspace://artifacts/audio_550e8400.wav",
      "encoding": "path",
      "metadata": {
        "sample_rate": 44100,
        "channels": 1
      }
    }
  ],
  "artifacts": [
    {
      "artifact_id": "990e8400-e29b-41d4-a716-446655440000",
      "kind": "file",
      "uri": "workspace://artifacts/audio_550e8400.wav",
      "sha256": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
      "size_bytes": 123456,
      "retention": "run"
    }
  ]
}
```

### fixtures/v1/responses/translation_success.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "succeeded",
  "timing": {
    "started_at": "2025-01-01T00:00:00Z",
    "finished_at": "2025-01-01T00:00:00.500Z",
    "duration_ms": 500
  },
  "outputs": [
    {
      "name": "translation_1",
      "content_type": "text/plain",
      "data": "Hola",
      "encoding": "utf-8",
      "metadata": {"index": 0}
    },
    {
      "name": "translation_2",
      "content_type": "text/plain",
      "data": "Mundo",
      "encoding": "utf-8",
      "metadata": {"index": 1}
    }
  ],
  "artifacts": []
}
```

### fixtures/v1/responses/simple_tts_error.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "failed",
  "timing": {
    "started_at": "2025-01-01T00:00:00Z",
    "finished_at": "2025-01-01T00:00:05.000Z",
    "duration_ms": 5000
  },
  "outputs": [],
  "artifacts": [],
  "error": {
    "code": "BACKEND_UNAVAILABLE",
    "message": "Text-to-speech service is temporarily unavailable",
    "retryable": true,
    "retry_after_ms": 5000,
    "retry_strategy": "exponential",
    "max_retries": 3,
    "details": {
      "backend_url": "http://localhost:7097",
      "http_status": 502,
      "last_successful": "2025-01-01T00:00:00Z"
    }
  }
}
```

### fixtures/v1/responses/validation_error.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440005",
  "status": "failed",
  "timing": {
    "started_at": "2025-01-01T00:00:00Z",
    "finished_at": "2025-01-01T00:00:00.010Z",
    "duration_ms": 10
  },
  "outputs": [],
  "artifacts": [],
  "error": {
    "code": "INVALID_INPUT_SCHEMA",
    "message": "Missing required field 'inputs'",
    "retryable": false,
    "details": {
      "field": "inputs",
      "reason": "required field missing"
    }
  }
}
```

### fixtures/v1/responses/async_video_accepted.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440003",
  "status": "accepted",
  "timing": {
    "accepted_at": "2025-01-01T00:00:00Z"
  },
  "outputs": [],
  "artifacts": [],
  "job": {
    "job_id": "aa0e8400-e29b-41d4-a716-446655440000",
    "state": "queued"
  }
}
```

### fixtures/v1/responses/async_video_completed.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440003",
  "status": "succeeded",
  "timing": {
    "accepted_at": "2025-01-01T00:00:00Z",
    "started_at": "2025-01-01T00:00:10Z",
    "finished_at": "2025-01-01T00:05:00Z",
    "duration_ms": 290000
  },
  "outputs": [
    {
      "name": "video",
      "content_type": "video/mp4",
      "data": "workspace://artifacts/video_aa0e8400.mp4",
      "encoding": "path",
      "metadata": {
        "duration_seconds": 10,
        "resolution": "1920x1080",
        "fps": 30
      }
    }
  ],
  "artifacts": [
    {
      "artifact_id": "bb0e8400-e29b-41d4-a716-446655440000",
      "kind": "file",
      "uri": "workspace://artifacts/video_aa0e8400.mp4",
      "sha256": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
      "size_bytes": 52428800,
      "retention": "pinned"
    }
  ],
  "job": {
    "job_id": "aa0e8400-e29b-41d4-a716-446655440000",
    "state": "succeeded"
  }
}
```

### fixtures/v1/responses/timeout_error.json

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440010",
  "status": "failed",
  "timing": {
    "started_at": "2025-01-01T00:00:00Z",
    "finished_at": "2025-01-01T00:10:00Z",
    "duration_ms": 600000
  },
  "outputs": [],
  "artifacts": [],
  "error": {
    "code": "TIMEOUT",
    "message": "Request exceeded timeout of 600000ms",
    "retryable": true,
    "retry_after_ms": 0,
    "retry_strategy": "immediate",
    "max_retries": 1,
    "details": {
      "timeout_ms": 600000,
      "elapsed_ms": 600000
    }
  }
}
```

## Acceptance Criteria

- [ ] Success response with outputs and artifacts
- [ ] Success response with inline data (no artifacts)
- [ ] Error response with retryable error
- [ ] Error response with non-retryable error
- [ ] Async accepted response with job info
- [ ] Async completed response
- [ ] All fixtures parse with `ESRPResponse`
- [ ] All fixtures pass validation

## Verification

```bash
# Parse with esrp-core
cargo test --package esrp-core parse_response_fixtures

# Or using CLI
esrp validate fixtures/v1/responses/simple_tts_success.json
```

## Notes

- Use fixed UUIDs matching the request fixtures
- Include realistic timing information
- Cover all status types: succeeded, failed, accepted
- Cover all error codes
- Use consistent workspace URIs with realistic paths
