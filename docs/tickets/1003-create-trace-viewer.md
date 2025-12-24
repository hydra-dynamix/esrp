# Ticket 1003: Create Trace Viewer

**Phase:** 10 - Event Logger
**Priority:** Low
**Dependencies:** [1002 - Integrate with Services](1002-integrate-with-services.md)
**Blocked By:** Ticket 1002

## Summary

Add CLI commands for viewing traces and events.

## Tasks

1. Add `esrp trace <trace_id>` command
2. Add `esrp request <request_id>` command
3. Format output for readability

## Implementation Details

```bash
esrp trace 550e8400-e29b-41d4-a716-446655440000

# Output:
Trace: 550e8400-e29b-41d4-a716-446655440000
Duration: 1234ms

┌─────────────────────────────────────────
│ 2025-01-01 00:00:00.000 service.requested
│   service: tts
│   operation: synthesize
│   request_id: 550e8400...
│
├─────────────────────────────────────────
│ 2025-01-01 00:00:01.234 service.completed
│   status: succeeded
│   duration: 1234ms
└─────────────────────────────────────────
```

## Acceptance Criteria

- [ ] Trace viewer works
- [ ] Human-readable output
- [ ] Can export as JSON
