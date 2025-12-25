# ESRP v1.0 Quick Reference Card

## Critical Design Decisions

### Canonical JSON Rules (NORMATIVE - MUST FOLLOW)
```
✓ Object keys: Sorted lexicographically by UTF-8 bytes
✓ Arrays: Preserve insertion order (do not sort)
✓ Numbers: Integers OK, FLOATS FORBIDDEN (use strings like "0.7")
✓ Whitespace: None (compact only)
✓ Encoding: UTF-8 only
✓ Booleans: lowercase true/false
✓ Null: lowercase null
```

**Why floats are banned:** Cross-platform serialization inconsistencies between Python, JS, Rust.

**Non-conformance:** Different canonical bytes for same input = protocol violation.

### Payload Hash Formula (AUTHORITATIVE)
```rust
payload_hash = sha256(canonical_json({
  "target": { service, operation, variant },
  "inputs": [...],
  "params": {...}
}))
```

**Include `target` to prevent cross-operation deduplication.**

**If idempotency_key is missing:** Server MUST compute it as `payload_hash` for side-effecting operations.

### Workspace URI Format
```
workspace://<namespace>/<path>

Examples:
  workspace://artifacts/audio_12345.wav
  workspace://temp/image.png

Rules:
  - Namespace: [a-zA-Z0-9._-]+, max 64 chars
  - Path: relative, no .., max 1024 chars
  - Must resolve through WorkspaceProvider
  - **Artifacts are IMMUTABLE once published**
```

**CRITICAL:** Never overwrite workspace URIs. Create new URIs for modified content.

### Causation vs Tracing
```
causation_id  → Why this request happened (coordination)
trace_id      → How this request executed (observability)
```

**Both are UUIDs, both are important, different purposes.**

### Idempotency
```
If idempotency_key provided:
  → Server MUST deduplicate
If idempotency_key absent:
  → Server SHOULD compute it as payload_hash
```

### Version Compatibility
```
Server accepts: 1.x
Client accepts: 1.x
Breaking change: 2.0
```

**Unknown fields MUST be ignored (forward compatibility).**

---

## Type System At-A-Glance

### Request Structure
```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "scope_id": "uuid",           // Unit of coherence
  "causation_id": "uuid",       // Parent request
  "payload_hash": "sha256",     // Dedup key
  
  "caller": {...},
  "target": { service, operation, variant },
  "mode": { type: "sync|async", timeout_ms: 600000 },
  "context": { trace_id, span_id, parent_span_id, tags },
  
  "inputs": [{ name, content_type, data, encoding, metadata }],
  "params": {}
}
```

### Response Structure
```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "status": "succeeded|failed|accepted",
  
  "timing": { accepted_at, started_at, finished_at, duration_ms },
  "outputs": [{ name, content_type, data, encoding, metadata }],
  "artifacts": [{ artifact_id, kind, uri, sha256, size_bytes, retention }],
  
  "job": { job_id, state },     // Only if status == "accepted"
  "error": { code, message, retryable, retry_after_ms, ... }
}
```

---

## Error Codes

### Infrastructure Errors (Retryable)
- `BACKEND_UNAVAILABLE` → 502 Bad Gateway
- `TIMEOUT` → 408 Request Timeout
- `OOM` → 507 Insufficient Storage

### Input Errors (Not Retryable)
- `INVALID_INPUT_SCHEMA` → 400 Bad Request
- `INVALID_INPUT_SEMANTIC` → 400 Bad Request
- `INVALID_INPUT_SIZE` → 413 Payload Too Large

**Always include `retry_after_ms` and `retry_strategy` for retryable errors.**

---

## Job State Machine (AUTHORITATIVE)

**Services implementing async MUST follow this FSM exactly. Deviations are non-conformant.**

```
Queued → Started → [Succeeded | Failed | Cancelled]
```

**Valid transitions:**
- Queued → Started, Cancelled
- Started → Succeeded, Failed, Cancelled

**Terminal states:** Succeeded, Failed, Cancelled (no further transitions allowed)

---

## HTTP Endpoints (Standard)

```
POST   /v1/execute              → Sync execution
POST   /v1/jobs                 → Async execution
GET    /v1/jobs/{job_id}        → Job status
GET    /v1/jobs/{job_id}/events → SSE stream
GET    /v1/capabilities         → Service metadata (reserved for v1.1+)
```

**Note:** `/v1/capabilities` is reserved for future capability discovery. Services MAY implement early, but clients MUST NOT require it in v1.0.

**Status code mapping:**
```
200 OK              → status: succeeded
202 Accepted        → status: accepted (async)
400 Bad Request     → INVALID_INPUT_*
408 Timeout         → TIMEOUT
502 Bad Gateway     → BACKEND_UNAVAILABLE
507 Insufficient    → OOM
```

---

## Retention Policies

```rust
Ephemeral → Delete after request completes
Run       → Delete when run_id/scope_id ends (default)
Pinned    → Never auto-delete (manual cleanup)
```

**Use `Ephemeral` for temp files, `Run` for session data, `Pinned` for outputs.**

---

## Crate Dependencies (One-Way)

```
esrp-core (no deps)
    ↑
esrp-canonical (depends on core)
    ↑
esrp-workspace (depends on core)
    ↑
esrp-http (depends on core, canonical, workspace)
```

**Rule:** Core stays pure, no circular deps.

---

## Conformance Testing Checklist

For each language binding:

- [ ] Parse all request fixtures
- [ ] Canonical JSON matches golden byte-for-byte
- [ ] SHA256 hashes match expected
- [ ] Round-trip: parse → canonicalize → parse == original
- [ ] Cross-language: Rust hash == Python hash == TS hash

**Fixtures must pass on Linux, macOS, Windows.**

---

## Common Pitfalls to Avoid

### ❌ DON'T
```rust
// Hand-roll JSON canonicalization
let json = format!("{}", value);

// Use floats in params that get hashed
"params": { "temperature": 0.7 }  // ❌ Non-deterministic

// Hardcode workspace paths
let path = "/tmp/artifacts/file.wav";

// Ignore version field
// (assume it's always 1.0)

// Mix causation and tracing
trace_id = parent_request_id  // ❌ Wrong
```

### ✅ DO
```rust
// Use canonical library
let canonical = esrp_canonical::to_canonical_json(&value)?;

// Stringify floats for hashing
"params": { "temperature": "0.7" }  // ✅ Deterministic

// Use workspace URIs
let uri = WorkspaceUri::parse("workspace://temp/file.wav")?;

// Validate version
if !is_compatible_version(&req.esrp_version) { reject(); }

// Keep causation and tracing separate
causation_id = parent_request.request_id
trace_id = new_uuid()
```

---

## Migration Strategy

### Single Server → Distributed

**Phase 1: Single Server**
- All services run locally
- HTTP calls via localhost
- SQLite event log

**Phase 2: Service Discovery**
- Add service registry
- Load balancing
- Health checks

**Phase 3: Event Stream**
- Replace SQLite with event bus (NATS/Kafka)
- Multiple writers
- Distributed tracing

**Phase 4: Synapsis**
- Event log becomes immutable append-only store
- Projections for current state
- Causation chain reconstruction

**ESRP stays the same across all phases.**

---

## Integration Examples

### Rust Service (Axum)
```rust
use esrp_http::extractors::ESRPRequestExtractor;

async fn execute(
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<Json<ESRPResponse>, StatusCode> {
    let text = request.inputs[0].data.clone();
    
    // Do work...
    
    Ok(Json(ESRPResponse {
        request_id: request.request_id,
        status: Status::Succeeded,
        outputs: vec![Output { ... }],
        // ...
    }))
}
```

### Python Client
```python
from esrp import ESRPClient, ESRPRequest, Input

client = ESRPClient("http://localhost:7097")

request = ESRPRequest(
    target=Target(service="tts", operation="synthesize"),
    inputs=[Input(name="text", data="Hello", content_type="text/plain")],
    params={"voice": "en-US-Standard-A"}
)

response = client.execute(request)
print(response.outputs[0].data)
```

### Python Service (FastAPI)
```python
from fastapi import FastAPI
from esrp.fastapi import esrp_middleware, esrp_response
from esrp.models import ESRPRequest, ESRPResponse

app = FastAPI()

@app.post("/v1/execute")
async def execute(request: ESRPRequest):
    # Validate via Rust
    esrp_py.validate_request(request.model_dump_json())
    
    # Do work...
    
    return esrp_response(ESRPResponse(
        request_id=request.request_id,
        status="succeeded",
        outputs=[...]
    ))
```

---

## When to Use What

### Use `scope_id` when:
- Grouping related requests (project, session, conversation)
- Tracking multi-step workflows
- Implementing cleanup policies

### Use `causation_id` when:
- One request triggers another
- Building dependency graphs
- Debugging "why did this happen?"

### Use `trace_id` when:
- Distributed tracing
- Performance monitoring
- Debugging "how did this execute?"

### Use `idempotency_key` when:
- Preventing duplicate side effects
- Retrying failed requests
- Ensuring exactly-once semantics

### Use `payload_hash` when:
- Deduplicating identical requests
- Caching results
- Auditing changes

---

## Testing Strategy

### Unit Tests
- Type validation
- Canonical JSON determinism
- Hash stability
- URI parsing edge cases

### Integration Tests
- Client/server round-trip
- Error handling
- Timeout behavior
- Async job lifecycle

### Conformance Tests
- Cross-language fixture parsing
- Canonical JSON byte-equality
- Hash consistency

### End-to-End Tests
- Multi-service orchestration
- Event log reconstruction
- Causation chain verification

---

## Performance Targets

```
Canonical JSON:     < 1ms for typical request
SHA256 hash:        < 1ms for typical payload
Workspace resolve:  < 10ms for local filesystem
HTTP round-trip:    < 100ms for local services
Event log write:    < 5ms for SQLite append
```

**These are soft targets for single-server deployment.**

---

## Security Considerations

### Input Validation
- Always validate `esrp_version`
- Reject oversized inputs (set limits)
- Sanitize workspace URIs (no `..` traversal)
- Validate artifact hashes before trusting

### Artifact Security
- Always verify SHA256 before using
- Use workspace URIs, never raw paths
- Enforce retention policies
- Scan artifacts for malware (if needed)

### Network Security
- Use HTTPS in production
- Validate Content-Type headers
- Rate limit per-service
- Log all requests

---

## Troubleshooting

### "Hash mismatch between Rust and Python"
→ Check for floats in params, convert to strings

### "Workspace URI not found"
→ Verify namespace exists, check retention policy

### "Version incompatible"
→ Ensure client/server both use 1.x

### "Idempotency key rejected"
→ Server computed different payload_hash, check inputs

### "Canonical JSON differs"
→ Check for extra whitespace, wrong key order, encoding issues

---

## Next Steps After v1.0

1. **TypeScript bindings** (WASM or codegen)
2. **Schema registry** for params validation
3. **Streaming** for large files
4. **Compression** (gzip/brotli)
5. **Multi-tenancy** (namespace isolation)
6. **Metrics** (Prometheus integration)
7. **Synapsis integration** (immutable event log)

---

## Resources

- **Spec:** `ESRP-SPEC.md`
- **Dev Plan:** `ESRP-DEVELOPMENT-PLAN.md`
- **Fixtures:** `fixtures/v1/`
- **CLI:** `esrp validate|canonicalize|hash`
- **Docs:** `cargo doc --open`

---

## Questions? Decision Points

### "Should I use sync or async mode?"
- Sync: < 10 seconds execution time
- Async: > 10 seconds or unpredictable duration

### "What retention policy should I use?"
- Ephemeral: Temp/scratch files
- Run: Session outputs
- Pinned: Important results

### "How do I handle large files?"
- Use `encoding: "path"` with workspace URI
- Store artifact separately
- Reference by SHA256

### "Can I extend ESRP with custom fields?"
- Yes, add to `params` or `metadata`
- Don't modify core schema
- Use `params_schema_ref` for validation

### "What if I need more than one output?"
- Use `outputs` array
- Each output has a `name`
- Or use `artifacts` for large outputs

---

**Remember:** ESRP is a coordination protocol, not a data format. Keep it simple, keep it typed, keep it immutable.
