# ESRP - Erasmus Service Request Protocol

A canonical messaging protocol designed for coordinating composable AI services with immutable, typed records.

## Overview

ESRP provides a foundation for multi-agent orchestration by ensuring:

- **Causality Tracking** - Explicit request chains via `causation_id`
- **Artifact Verification** - Content-addressed storage with SHA256 hashing
- **Idempotency** - Payload hashing for deduplication and replay
- **Distributed Tracing** - Built-in observability support
- **Job Lifecycle** - Async operation management with defined state machine

## Design Principles

1. **No Implicit State** - Every execution is self-contained
2. **Artifact Immutability** - Files referenced by cryptographic hash
3. **Replay-ability** - Complete system state reconstruction from event log
4. **Protocol-First** - Types enforce coordination discipline
5. **Transport-Agnostic** - Works over HTTP, event streams, or message queues

## Architecture

```
Orchestrators → Decide what should happen (planning)
Agents        → Reason about context (interpretation)
Services      → Execute operations (tools)
Synapsis      → Remember everything (immutable log)
```

ESRP sits between services and the event log, providing the canonical wire format.

## Project Structure

```
esrp/
├── crates/
│   ├── esrp-core/        # Core types and validation (zero deps)
│   ├── esrp-canonical/   # Deterministic JSON serialization & hashing
│   ├── esrp-workspace/   # Workspace URI handling & artifact storage
│   └── esrp-http/        # Axum handlers & Reqwest client
├── bindings/
│   ├── python/           # PyO3 bindings + Pydantic models
│   └── typescript/       # WASM or codegen bindings (future)
├── fixtures/
│   └── v1/               # Golden test fixtures
├── tools/
│   └── esrp-cli/         # Command-line tool
└── docs/
    ├── ESRP-SPEC.md              # Full protocol specification
    ├── ESRP-DEVELOPMENT-PLAN.md  # Implementation roadmap
    └── ESRP-QUICK-REFERENCE.md   # Developer cheat sheet
```

## Quick Start

### Request Structure

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-01-01T00:00:00Z",
  "caller": { "system": "erasmus" },
  "target": { "service": "tts", "operation": "synthesize" },
  "mode": { "type": "sync", "timeout_ms": 60000 },
  "inputs": [{
    "name": "text",
    "content_type": "text/plain",
    "data": "Hello world",
    "encoding": "utf-8"
  }],
  "params": { "voice": "en-US-Standard-A" }
}
```

### Response Structure

```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "succeeded",
  "timing": {
    "started_at": "2025-01-01T00:00:00Z",
    "finished_at": "2025-01-01T00:00:01Z",
    "duration_ms": 1234.5
  },
  "outputs": [{
    "name": "audio",
    "content_type": "audio/wav",
    "data": "workspace://artifacts/audio_12345.wav",
    "encoding": "path"
  }],
  "artifacts": [{
    "artifact_id": "...",
    "uri": "workspace://artifacts/audio_12345.wav",
    "sha256": "abc123...",
    "size_bytes": 12345,
    "retention": "run"
  }]
}
```

## Key Concepts

### Coordination Fields

| Field | Purpose |
|-------|---------|
| `scope_id` | Groups related requests (project/task/thread) |
| `causation_id` | Links to parent request that triggered this one |
| `payload_hash` | SHA256 of canonical `{target, inputs, params}` for dedup |
| `trace_id` | Distributed tracing for observability |

### Canonical JSON Rules

ESRP requires deterministic JSON serialization for hashing:

- Object keys sorted lexicographically
- Arrays preserve insertion order
- **Floats forbidden** - use strings like `"0.7"` instead
- No whitespace, UTF-8 only

### Job State Machine

```
Queued → Started → [Succeeded | Failed | Cancelled]
```

### Error Codes

| Code | HTTP | Retryable |
|------|------|-----------|
| `BACKEND_UNAVAILABLE` | 502 | Yes |
| `TIMEOUT` | 408 | Yes |
| `OOM` | 507 | Yes |
| `INVALID_INPUT_SCHEMA` | 400 | No |
| `INVALID_INPUT_SEMANTIC` | 400 | No |

## HTTP Endpoints

```
POST   /v1/execute              → Sync execution
POST   /v1/jobs                 → Async execution
GET    /v1/jobs/{job_id}        → Job status
GET    /v1/jobs/{job_id}/events → SSE stream
```

## Development

### Building

```bash
cargo build --workspace
cargo test --workspace
```

### CLI Tool

```bash
cargo install --path tools/esrp-cli

esrp validate request.json      # Validate ESRP JSON
esrp canonicalize request.json  # Output canonical JSON
esrp hash request.json          # Compute payload hash
```

### Conformance Testing

```bash
cargo test --package esrp-core conformance
pytest bindings/python/tests/conformance/
```

## Documentation

- [Full Specification](docs/ESRP-SPEC.md) - Complete protocol details
- [Development Plan](docs/ESRP-DEVELOPMENT-PLAN.md) - Implementation roadmap
- [Quick Reference](docs/ESRP-QUICK-REFERENCE.md) - Developer cheat sheet

## Status

**Version:** 1.0 (Draft)

This protocol is under active development. See the [Development Plan](docs/ESRP-DEVELOPMENT-PLAN.md) for current progress.

## License

[Add license information]
