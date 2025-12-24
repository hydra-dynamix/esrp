# ESRP v1.0 Specification
## Erasmus Service Request Protocol

**Version:** 1.0  
**Status:** Draft  
**Last Updated:** 2025-12-22

---

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Type System](#type-system)
4. [Canonical Representation](#canonical-representation)
5. [Workspace Semantics](#workspace-semantics)
6. [Job Lifecycle](#job-lifecycle)
7. [Error Model](#error-model)
8. [Versioning](#versioning)
9. [Conformance](#conformance)

---

## Overview

ESRP is a protocol for coordinating composable AI services with immutable, typed records. It provides:

- **Causality tracking** through request chains
- **Artifact verification** via content-addressed storage
- **Idempotency** through payload hashing
- **Distributed tracing** for observability
- **Job lifecycle** management for async operations

### Design Goals

1. **No implicit state** - Every execution is self-contained
2. **Artifact immutability** - Files referenced by cryptographic hash
3. **Replay-ability** - Complete system state reconstruction from event log
4. **Protocol-first** - Types enforce coordination discipline
5. **Transport-agnostic** - Works over HTTP, event streams, or message queues

---

## Core Principles

### Decoherence as Protocol Failure

Multi-agent systems fail when coordination relies on implicit state (chat history, memory, "it said it did that"). ESRP prevents decoherence by:

- Externalizing all coordination state into typed records
- Making causality explicit through `causation_id` chains
- Using content hashes for artifact claims
- Separating execution (services) from memory (event log)

### Separation of Concerns

```
Orchestrators → Decide what should happen (planning)
Agents        → Reason about context (interpretation)
Services      → Execute operations (tools)
Synapsis      → Remember everything (immutable log)
```

ESRP sits between services and Synapsis, providing the canonical wire format.

---

## Type System

### Core Request Structure

```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "idempotency_key": "optional-hash",
  "timestamp": "RFC3339",
  
  "scope_id": "uuid-optional",
  "causation_id": "uuid-optional",
  "payload_hash": "sha256-optional",
  
  "caller": {
    "system": "erasmus",
    "agent_id": "optional",
    "run_id": "optional"
  },
  
  "target": {
    "service": "art|translator|tts|stt|video|tokenizer",
    "operation": "generate|translate|synthesize|transcribe",
    "variant": "optional"
  },
  
  "mode": {
    "type": "sync|async",
    "timeout_ms": 600000
  },
  
  "context": {
    "trace_id": "uuid",
    "span_id": "uuid",
    "parent_span_id": "uuid-optional",
    "tags": {}
  },
  
  "inputs": [
    {
      "name": "text|audio|image",
      "content_type": "text/plain|audio/wav|image/png",
      "data": "...",
      "encoding": "utf-8|base64|path",
      "metadata": {}
    }
  ],
  
  "params": {},
  "params_schema_ref": "optional-url"
}
```

### Core Response Structure

```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "status": "succeeded|failed|accepted",
  
  "timing": {
    "accepted_at": "RFC3339",
    "started_at": "RFC3339",
    "finished_at": "RFC3339",
    "duration_ms": 1234.5
  },
  
  "outputs": [
    {
      "name": "result",
      "content_type": "text/plain",
      "data": "...",
      "encoding": "utf-8",
      "metadata": {}
    }
  ],
  
  "artifacts": [
    {
      "artifact_id": "uuid",
      "kind": "file|blob",
      "uri": "workspace://namespace/path",
      "sha256": "hex-string",
      "size_bytes": 12345,
      "retention": "ephemeral|run|pinned"
    }
  ],
  
  "job": {
    "job_id": "uuid",
    "state": "queued|started|succeeded|failed|cancelled"
  },
  
  "error": {
    "code": "BACKEND_UNAVAILABLE|INVALID_INPUT_SCHEMA|...",
    "message": "human-readable",
    "retryable": true,
    "retry_after_ms": 5000,
    "retry_strategy": "exponential|linear|immediate",
    "max_retries": 3,
    "details": {}
  }
}
```

### Field Semantics

#### Coordination Fields

- **`scope_id`**: The unit of coherence (project/task/thread). All requests in the same scope share context.
- **`causation_id`**: Links to the `request_id` of the parent request that caused this one. Forms the causation chain.
- **`payload_hash`**: SHA256 hash of canonical JSON of `{inputs, params, target}`. Used for idempotency and deduplication.

#### Tracing vs Causation

- **`trace_id`**: Distributed trace for observability (how the request executed)
- **`causation_id`**: Coordination chain (why the request happened)

These are distinct concepts. A single trace may span multiple causation chains, or vice versa.

#### Idempotency

- If `idempotency_key` is provided, server MUST deduplicate requests with the same key
- If `idempotency_key` is absent for side-effecting operations, server SHOULD compute it as `payload_hash`
- Server MAY reject requests where provided `idempotency_key` doesn't match computed `payload_hash`

---

## Canonical Representation

### Requirements

All ESRP implementations MUST support canonical JSON serialization for:

- Computing `payload_hash`
- Deriving `idempotency_key`
- Event log anchoring
- Artifact verification

### Normative Canonical JSON Rules

**These rules are NORMATIVE and MUST be followed by all ESRP implementations.**

All ESRP implementations MUST produce byte-identical canonical JSON for the same logical input. Canonical JSON is defined as follows:

1. **Object keys**: MUST be sorted lexicographically by UTF-8 byte values (Unicode code points)
2. **Arrays**: MUST preserve insertion order (do not sort)
3. **Strings**: MUST use standard JSON escape rules per RFC 8259
4. **Numbers**: 
   - Integers: MUST be decimal representation with no leading zeros (except `0` itself)
   - Floats: **MUST NOT** appear in canonical-hashed regions. Implementations MUST represent floating-point values as strings (e.g., `"0.7"` not `0.7`)
5. **Whitespace**: MUST NOT include any insignificant whitespace (no spaces, newlines, or indentation)
6. **Character encoding**: MUST be UTF-8 only
7. **Formatting**: MUST be compact (no newlines, no indentation, no spaces after `:` or `,`)
8. **Boolean values**: MUST be lowercase `true` or `false`
9. **Null values**: MUST be lowercase `null`

**Rationale for float restriction:** Different platforms (Python, JavaScript, Rust) serialize floating-point numbers inconsistently due to IEEE 754 representation differences. Using string representation ensures cross-platform hash stability.

**Compliance:** Any implementation producing different canonical bytes for the same input is non-conformant and will produce incorrect payload hashes.

### Number Handling

To prevent cross-platform hash inconsistencies:

- **Rule**: Floating-point numbers MUST be represented as strings in fields that contribute to `payload_hash`
- **Rationale**: Different platforms (Python, JavaScript, Rust) serialize floats inconsistently
- **Example**: Use `"temperature": "0.7"` not `"temperature": 0.7` in `params` if hashing

Integers are safe to represent as JSON numbers.

### Payload Hash Computation

**AUTHORITATIVE DEFINITION:**

The `payload_hash` field is computed as follows:

```
payload_hash = sha256(canonical_json({
  "target": {
    "service": "...",
    "operation": "...",
    "variant": "..." | null
  },
  "inputs": [...],
  "params": {...}
}))
```

**Why include `target`**: Prevents accidental deduplication when the same input is sent to different operations.

### Idempotency Key Derivation

If `idempotency_key` is not provided by the client and the operation is side-effecting (creates, updates, or deletes state), the server MUST compute it as:

```
idempotency_key = payload_hash
```

The server MAY reject requests where the provided `idempotency_key` does not match the computed `payload_hash` (strict validation mode).

Servers implementing idempotency MUST:
1. Store the mapping `idempotency_key → (request_id, response)` 
2. Return the cached response for duplicate keys
3. Ensure the response includes the original `request_id`

**Deduplication window:** Servers SHOULD maintain idempotency mappings for at least the `timeout_ms` of the original request, and MAY extend this indefinitely.

### Reference Implementation

The canonical implementation is in `esrp-canonical` (Rust crate). All language bindings MUST produce byte-identical output for the same input.

---

## Workspace Semantics

### URI Format

```
workspace://<namespace>/<path>
```

**Examples:**
- `workspace://artifacts/audio_12345.wav`
- `workspace://temp/image.png`
- `workspace://runs/run-abc/output.json`

### Namespace Rules

- **Character set**: `[a-zA-Z0-9._-]+` (alphanumeric, dot, underscore, hyphen)
- **Max length**: 64 characters
- **Reserved namespaces**: `system`, `tmp`, `cache`

### Path Rules

- **Must be relative**: No leading `/`, no `..` traversal
- **Character encoding**: UTF-8 with percent-encoding for special characters
- **Max length**: 1024 characters
- **Case sensitivity**: Preserve case, but comparison is case-sensitive

### Retention Policies

```rust
enum RetentionPolicy {
    Ephemeral,  // Delete after request completes
    Run,        // Keep for duration of run/session (default)
    Pinned,     // Keep indefinitely
}
```

- **Ephemeral**: Deleted when response is sent
- **Run**: Deleted when `run_id` or `scope_id` session ends
- **Pinned**: Never automatically deleted (requires explicit cleanup)

### Artifact Requirements

All `Artifact` records MUST include:

1. **`uri`**: Valid workspace URI
2. **`sha256`**: Hex-encoded SHA256 hash of content (64 characters)
3. **`size_bytes`**: Exact byte size (u64)
4. **`kind`**: `file` (filesystem) or `blob` (object store)

### Artifact Immutability

**CRITICAL RULE:** Artifacts referenced by workspace URIs are **immutable** once published.

Services MUST NOT:
- Overwrite existing artifacts
- Modify artifact content in-place
- Reuse URIs for different content

Services MUST:
- Verify SHA256 hash before using artifacts
- Reject artifacts where `sha256` verification fails
- Create new URIs for modified content

**Rationale:** Artifact immutability is essential for:
- Multi-agent trust (agents can verify claims)
- Causation chain reconstruction (history is stable)
- Event log integrity (references remain valid)

**Violation handling:** If a service detects an artifact hash mismatch, it MUST:
1. Log the violation (artifact_id, expected_hash, actual_hash)
2. Return an error response with code `INVALID_INPUT_SEMANTIC`
3. Not proceed with execution

**Workspace provider enforcement:** Workspace implementations SHOULD enforce write-once semantics at the storage layer when possible.

### Workspace Provider Trait

Services MUST NOT hardcode filesystem paths. Instead, they MUST resolve workspace URIs through a `WorkspaceProvider`:

```rust
trait WorkspaceProvider {
    fn resolve(&self, namespace: &str, path: &Path) -> Result<PathBuf>;
    fn store(&self, namespace: &str, data: &[u8]) -> Result<String>;
}
```

This allows workspace backends to be:
- Local filesystem
- S3/object storage
- Content-addressed store
- Distributed filesystem

---

## Job Lifecycle

### State Machine

**AUTHORITATIVE JOB FSM:**

Services implementing async execution (`mode.type = "async"`) MUST emit job state transitions that follow this state machine exactly. Deviations from this FSM are non-conformant.

```
┌─────────┐
│ Queued  │
└────┬────┘
     │
     v
┌─────────┐
│ Started │
└────┬────┘
     │
     ├──────────┬─────────────┐
     v          v             v
┌──────────┐ ┌────────┐ ┌───────────┐
│Succeeded │ │ Failed │ │ Cancelled │
└──────────┘ └────────┘ └───────────┘
```

**Valid Transitions (NORMATIVE):**
- `Queued → Started` (job begins execution)
- `Queued → Cancelled` (job cancelled before starting)
- `Started → Succeeded` (job completed successfully)
- `Started → Failed` (job failed with error)
- `Started → Cancelled` (job cancelled during execution)

**Invalid Transitions:**
- Cannot transition FROM terminal states (`Succeeded`, `Failed`, `Cancelled`)
- Cannot skip `Started` state when transitioning to terminal states from `Queued`
- No other transitions are permitted

**Enforcement:** Servers MUST reject any state transition request that violates these rules with `INVALID_INPUT_SEMANTIC` error.

### Job Events

```rust
enum JobEventType {
    JobQueued,
    JobStarted,
    JobProgress,      // Optional progress updates
    ArtifactCreated,  // When intermediate artifacts are produced
    JobCompleted,
    JobFailed,
    JobCancelled,
}
```

Each event MUST include:
- `event_type`: One of the above types
- `job_id`: UUID of the job
- `timestamp`: RFC3339 timestamp
- `data`: Event-specific payload

### Async Response Flow

1. Client sends `ESRPRequest` with `mode.type = "async"`
2. Server returns `ESRPResponse` with:
   - `status = "accepted"`
   - `job.job_id = <uuid>`
   - `job.state = "queued"`
3. Client polls `GET /jobs/{job_id}` or subscribes to `GET /jobs/{job_id}/events` (SSE)
4. Server emits `JobEvent` records as job progresses
5. Final event is `JobCompleted` (with full `ESRPResponse`) or `JobFailed` (with error)

### Job Reference in Response

When `status == "accepted"`, the response MUST include:

```json
"job": {
  "job_id": "uuid",
  "state": "queued"
}
```

When `status == "succeeded"` or `status == "failed"`, the `job` field MAY be omitted (sync execution) or MAY be included (completed async job).

---

## Error Model

### Error Codes

```rust
enum ErrorCode {
    // Infrastructure
    BACKEND_UNAVAILABLE,      // Service temporarily down
    TIMEOUT,                   // Request exceeded timeout_ms
    OOM,                       // Out of memory
    
    // Input validation
    INVALID_INPUT_SCHEMA,      // Input structure invalid
    INVALID_INPUT_SEMANTIC,    // Input valid but request impossible
    INVALID_INPUT_SIZE,        // Input exceeds size limits
    
    // Unknown
    UNKNOWN,                   // Catch-all
}
```

### Error Response Structure

All errors MUST be structured, never plain text.

```json
{
  "code": "BACKEND_UNAVAILABLE",
  "message": "Text-to-speech backend not responding",
  "retryable": true,
  "retry_after_ms": 5000,
  "retry_strategy": "exponential",
  "max_retries": 3,
  "details": {
    "backend_url": "http://localhost:7097",
    "http_status": 502
  }
}
```

### Retry Strategy Semantics

- **`exponential`**: Backoff with exponential growth (e.g., 1s, 2s, 4s, 8s)
- **`linear`**: Fixed backoff interval (e.g., 5s, 5s, 5s)
- **`immediate`**: Retry without delay (use sparingly)

### Retryable Errors

The following errors SHOULD be marked `retryable: true`:

- `BACKEND_UNAVAILABLE` (temporary network/service issues)
- `TIMEOUT` (if operation is idempotent)
- `OOM` (if request can be chunked or retried with fewer resources)

The following errors SHOULD be marked `retryable: false`:

- `INVALID_INPUT_SCHEMA` (client bug, won't fix itself)
- `INVALID_INPUT_SEMANTIC` (impossible request)
- `INVALID_INPUT_SIZE` (exceeds hard limits)

### Error Logging

All errors MUST be logged as `service.failed` events with:
- Full `ESRPRequest` (for replay)
- Full `Error` object
- Stack trace or debug info in `details`

---

## Versioning

### Version Format

```
<major>.<minor>
```

**Examples:** `1.0`, `1.1`, `2.0`

### Compatibility Rules

- **Major version change**: Breaking changes (incompatible)
- **Minor version change**: Backward-compatible additions

**ESRP 1.x Compatibility:**
- All `1.x` versions MUST accept `1.0` requests
- Clients SHOULD accept any `1.x` response
- Unknown fields MUST be ignored (forward compatibility)

### Version Validation

Servers MUST:
1. Parse `esrp_version` from request
2. Check major version equals `1`
3. Reject requests with `major != 1` with `INVALID_INPUT_SCHEMA` error

### Version Struct

Instead of raw string parsing, implementations SHOULD use a typed version struct:

```rust
struct ESRPVersion {
    major: u8,
    minor: u8,
}

impl ESRPVersion {
    fn is_compatible_with(&self, other: &ESRPVersion) -> bool {
        self.major == other.major
    }
}
```

---

## Conformance

### Test Fixtures

All implementations MUST pass the golden fixtures in `fixtures/v1/`:

```
fixtures/v1/
├── requests/
│   ├── simple_tts.json
│   ├── batch_translation.json
│   └── image_generation.json
├── canonical/
│   ├── simple_tts.json          # Canonical JSON output
│   └── simple_tts.sha256         # Expected hash
└── responses/
    ├── simple_tts_success.json
    └── simple_tts_error.json
```

### Test Requirements

1. **Parse**: Deserialize request JSON → type-safe structure
2. **Canonicalize**: Serialize to canonical JSON → byte-identical to fixture
3. **Hash**: Compute SHA256 → matches expected hash
4. **Round-trip**: Parse → Canonicalize → Parse → equal to original

### Cross-Language Conformance

The Rust implementation is the reference. Python and TypeScript bindings MUST:

- Parse all fixtures without error
- Produce byte-identical canonical JSON
- Produce identical SHA256 hashes

### Conformance Test Suite

```bash
# Run conformance tests
cargo test --package esrp-core conformance
pytest tests/conformance/
npm test -- conformance
```

All three MUST pass before a release.

---

## Transport Independence

ESRP is transport-agnostic. The core protocol makes no assumptions about HTTP, gRPC, or message queues.

### HTTP Binding (Recommended)

**Endpoints:**
- `POST /v1/execute` → Sync execution
- `POST /v1/jobs` → Async execution
- `GET /v1/jobs/{job_id}` → Job status
- `GET /v1/jobs/{job_id}/events` → SSE event stream
- `GET /v1/capabilities` → Service capabilities (reserved)

**Headers:**
- `Content-Type: application/json`
- `X-Request-ID: {uuid}` (optional, mirrors `request_id`)
- `X-Idempotency-Key: {hash}` (optional, mirrors `idempotency_key`)

**Status Codes:**
- `200 OK` → Success
- `202 Accepted` → Async job queued
- `400 Bad Request` → Invalid input
- `408 Request Timeout` → Timeout
- `502 Bad Gateway` → Backend unavailable
- `507 Insufficient Storage` → OOM or disk full

---

## Capability Discovery (Reserved)

**Status:** Reserved for v1.1+

ESRP reserves the `/v1/capabilities` endpoint for service discovery. While not required for v1.0, implementations SHOULD reserve this endpoint to avoid future conflicts.

### Proposed Capabilities Schema (Informational)

```json
{
  "service": "tts",
  "version": "1.0.2",
  "esrp_version": "1.0",
  "operations": [
    {
      "name": "synthesize",
      "description": "Convert text to speech",
      "supports_async": true,
      "supports_batch": false,
      "input_schema_ref": "https://example.com/schemas/tts-input.json",
      "params_schema_ref": "https://example.com/schemas/tts-params.json",
      "output_schema_ref": "https://example.com/schemas/tts-output.json"
    }
  ],
  "workspace_namespaces": ["audio", "temp"],
  "rate_limits": {
    "requests_per_minute": 60,
    "max_input_size_bytes": 1048576
  }
}
```

**Why reserved:** Orchestrators currently hardcode service knowledge. Discovery will enable dynamic service composition without updating orchestrator code.

**v1.0 conformance:** Services MAY implement this endpoint early, but clients MUST NOT require it.

### Event Stream Binding

ESRP requests/responses can be serialized directly into event logs (Kafka, NATS, Synapsis) with no modifications.

**Event Types:**
- `service.requested` → Full `ESRPRequest`
- `service.completed` → Full `ESRPResponse` (status: succeeded)
- `service.failed` → Full `ESRPResponse` (status: failed)
- `artifact.created` → Artifact record
- `job.queued` → Job event
- `job.started` → Job event
- `job.completed` → Job event

---

## Implementation Checklist

### Core Protocol (esrp-core)
- [ ] Define all types in Rust
- [ ] Implement version validation
- [ ] Implement input validation
- [ ] Derive serialization (serde)
- [ ] Write unit tests for types

### Canonicalization (esrp-canonical)
- [ ] Implement canonical JSON serialization
- [ ] Implement SHA256 hashing
- [ ] Implement `derive_payload_hash()`
- [ ] Write tests for determinism
- [ ] Document number handling rules

### Workspace (esrp-workspace)
- [ ] Implement URI parser
- [ ] Define `WorkspaceProvider` trait
- [ ] Implement `FilesystemWorkspace`
- [ ] Implement artifact store/retrieve
- [ ] Write tests for URI edge cases

### HTTP Transport (esrp-http)
- [ ] Implement Axum extractors
- [ ] Implement Reqwest client
- [ ] Implement error mapping
- [ ] Implement job endpoints (async)
- [ ] Write integration tests

### Language Bindings
- [ ] Python (PyO3) bindings
- [ ] TypeScript (WASM or codegen) bindings
- [ ] Client libraries (sync/async)
- [ ] Server middleware

### Conformance
- [ ] Create golden fixtures
- [ ] Implement cross-language tests
- [ ] Document expected behavior
- [ ] CI pipeline for all languages

---

## Future Extensions (Not in v1.0)

- **Streaming**: Chunked inputs/outputs for large files
- **Schema registry**: Centralized params schema validation
- **Multi-tenancy**: Namespace isolation and quotas
- **Compression**: gzip/brotli for large payloads
- **Authentication**: Token-based auth headers
- **Rate limiting**: Per-service/per-user quotas

These are explicitly deferred to maintain v1.0 simplicity.

---

## References

- [RFC 8259: JSON](https://tools.ietf.org/html/rfc8259)
- [RFC 3339: Date/Time](https://tools.ietf.org/html/rfc3339)
- [RFC 8785: JCS (Canonical JSON)](https://tools.ietf.org/html/rfc8785)
- [Distributed Tracing: W3C Trace Context](https://www.w3.org/TR/trace-context/)
