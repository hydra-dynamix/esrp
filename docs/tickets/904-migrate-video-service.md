# Ticket 904: Migrate Video Service

**Phase:** 9 - Service Migration
**Priority:** Medium
**Dependencies:** [903 - Migrate STT Service](903-migrate-stt-service.md)
**Blocked By:** Ticket 903

## Summary

Migrate the video generation service to ESRP format with async job support.

## Special Considerations

- Video generation is long-running, needs async mode
- Must implement job state machine (queued → started → succeeded/failed)
- Must emit job events

## Acceptance Criteria

- [ ] Service returns `status: accepted` for async requests
- [ ] Job state transitions follow FSM
- [ ] Completed job includes video artifact
