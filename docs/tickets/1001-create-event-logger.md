# Ticket 1001: Create Event Logger

**Phase:** 10 - Event Logger
**Priority:** Medium
**Dependencies:** Phase 5 completion
**Blocked By:** Conformance tests passing

## Summary

Implement a simple SQLite-based event logger for ESRP requests and responses.

## Tasks

1. Define event schema
2. Implement EventLog struct
3. Implement log methods
4. Implement query methods

## Implementation Details

### Schema

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    request_id TEXT NOT NULL,
    trace_id TEXT,
    scope_id TEXT,
    causation_id TEXT,
    service TEXT,
    operation TEXT,
    status TEXT,
    duration_ms REAL,
    payload TEXT NOT NULL
);

CREATE INDEX idx_request_id ON events(request_id);
CREATE INDEX idx_trace_id ON events(trace_id);
CREATE INDEX idx_timestamp ON events(timestamp);
```

### EventLog

```rust
pub struct EventLog {
    db: rusqlite::Connection,
}

impl EventLog {
    pub fn new(db_path: &Path) -> Result<Self>;
    pub fn log_request(&self, req: &ESRPRequest) -> Result<i64>;
    pub fn log_response(&self, req: &ESRPRequest, res: &ESRPResponse) -> Result<i64>;
    pub fn get_trace(&self, trace_id: Uuid) -> Result<Vec<Event>>;
}
```

## Acceptance Criteria

- [ ] Can log requests and responses
- [ ] Can query by trace_id
- [ ] Can query by request_id
- [ ] Events are append-only
