# Ticket 902: Migrate TTS Service

**Phase:** 9 - Service Migration
**Priority:** Medium
**Dependencies:** [901 - Migrate Translator Service](901-migrate-translator-service.md)
**Blocked By:** Ticket 901 (for pattern reference)

## Summary

Migrate the text-to-speech service to ESRP format, following the pattern from translator migration.

## Acceptance Criteria

- [ ] Service accepts ESRP requests
- [ ] Service returns ESRP responses with artifacts
- [ ] Audio files stored via WorkspaceProvider
- [ ] SHA256 hashes included in artifacts
