# ESRP Implementation Tickets

This directory contains actionable implementation tickets derived from the ESRP Development Plan. Each ticket is designed to be picked up independently with minimal context overhead.

## Ticket Numbering

- `0XX` - Phase 0: Repository Setup
- `1XX` - Phase 1: Core Protocol Types
- `2XX` - Phase 2: Canonical Representation
- `3XX` - Phase 3: Workspace Semantics
- `4XX` - Phase 4: Test Fixtures
- `5XX` - Phase 5: Conformance Tests
- `6XX` - Phase 6: HTTP Transport
- `7XX` - Phase 7: CLI Tool
- `8XX` - Phase 8: Python Bindings
- `9XX` - Phase 9: Service Migration
- `10XX` - Phase 10: Event Logger

## Dependencies

```
Phase 0 (Setup)
    ↓
Phase 1 (Core Types)
    ↓
    ├─→ Phase 2 (Canonical) ─→ Phase 4 (Fixtures) ─→ Phase 5 (Conformance)
    │                                                        ↓
    └─→ Phase 3 (Workspace) ─────────────────────────→ Phase 6 (HTTP)
                                                             ↓
                                                       Phase 7 (CLI)
                                                             ↓
                                                       Phase 8 (Python)
                                                             ↓
                                                       Phase 9 (Migration)
                                                             ↓
                                                       Phase 10 (Event Log)
```

## Ticket Status Legend

- `[ ]` - Not started
- `[~]` - In progress
- `[x]` - Complete

## All Tickets

### Phase 0: Repository Setup
- [x] [001 - Create Cargo Workspace](001-create-cargo-workspace.md)
- [ ] [002 - Initialize Git Repository](002-initialize-git-repository.md)

### Phase 1: Core Protocol Types
- [x] [101 - Create esrp-core Crate](101-create-esrp-core-crate.md)
- [x] [102 - Implement Core Types](102-implement-core-types.md)
- [x] [103 - Implement Version Validation](103-implement-version-validation.md)
- [x] [104 - Implement Input Validation](104-implement-input-validation.md)
- [x] [105 - Write Core Unit Tests](105-write-core-unit-tests.md)

### Phase 2: Canonical Representation
- [ ] [201 - Create esrp-canonical Crate](201-create-esrp-canonical-crate.md)
- [ ] [202 - Implement Canonical JSON Serialization](202-implement-canonical-json.md)
- [ ] [203 - Implement SHA256 Hashing](203-implement-hashing.md)
- [ ] [204 - Implement Payload Hash Derivation](204-implement-payload-hash.md)
- [ ] [205 - Document Number Handling](205-document-number-handling.md)
- [ ] [206 - Write Canonical Tests](206-write-canonical-tests.md)

### Phase 3: Workspace Semantics
- [ ] [301 - Create esrp-workspace Crate](301-create-esrp-workspace-crate.md)
- [ ] [302 - Implement URI Parser](302-implement-uri-parser.md)
- [ ] [303 - Define WorkspaceProvider Trait](303-define-workspace-provider-trait.md)
- [ ] [304 - Implement FilesystemWorkspace](304-implement-filesystem-workspace.md)
- [ ] [305 - Write Workspace Tests](305-write-workspace-tests.md)

### Phase 4: Test Fixtures
- [ ] [401 - Create Fixture Directory Structure](401-create-fixture-structure.md)
- [ ] [402 - Create Request Fixtures](402-create-request-fixtures.md)
- [ ] [403 - Generate Canonical Fixtures](403-generate-canonical-fixtures.md)
- [ ] [404 - Create Response Fixtures](404-create-response-fixtures.md)
- [ ] [405 - Document Fixture Format](405-document-fixture-format.md)

### Phase 5: Conformance Tests
- [ ] [501 - Create Conformance Test Suite](501-create-conformance-test-suite.md)
- [ ] [502 - Add CI Pipeline](502-add-ci-pipeline.md)

### Phase 6: HTTP Transport
- [ ] [601 - Create esrp-http Crate](601-create-esrp-http-crate.md)
- [ ] [602 - Implement Axum Extractors](602-implement-axum-extractors.md)
- [ ] [603 - Implement Response Helpers](603-implement-response-helpers.md)
- [ ] [604 - Implement Reqwest Client](604-implement-reqwest-client.md)
- [ ] [605 - Write HTTP Integration Tests](605-write-http-integration-tests.md)

### Phase 7: CLI Tool
- [ ] [701 - Create esrp-cli Crate](701-create-esrp-cli-crate.md)
- [ ] [702 - Implement CLI Subcommands](702-implement-cli-subcommands.md)
- [ ] [703 - Write CLI Tests](703-write-cli-tests.md)

### Phase 8: Python Bindings
- [ ] [801 - Create PyO3 Bindings](801-create-pyo3-bindings.md)
- [ ] [802 - Create Pydantic Models](802-create-pydantic-models.md)
- [ ] [803 - Create Python Client](803-create-python-client.md)
- [ ] [804 - Create FastAPI Middleware](804-create-fastapi-middleware.md)
- [ ] [805 - Write Python Tests](805-write-python-tests.md)

### Phase 9: Service Migration
- [ ] [901 - Migrate Translator Service](901-migrate-translator-service.md)
- [ ] [902 - Migrate TTS Service](902-migrate-tts-service.md)
- [ ] [903 - Migrate STT Service](903-migrate-stt-service.md)
- [ ] [904 - Migrate Video Service](904-migrate-video-service.md)
- [ ] [905 - Update Python Services](905-update-python-services.md)

### Phase 10: Event Logger
- [ ] [1001 - Create Event Logger](1001-create-event-logger.md)
- [ ] [1002 - Integrate with Services](1002-integrate-with-services.md)
- [ ] [1003 - Create Trace Viewer](1003-create-trace-viewer.md)
