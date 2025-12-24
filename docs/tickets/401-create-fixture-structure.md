# Ticket 401: Create Fixture Directory Structure

**Phase:** 4 - Test Fixtures
**Priority:** High
**Dependencies:** [201 - Create esrp-canonical Crate](201-create-esrp-canonical-crate.md)
**Blocked By:** Phase 2 completion

## Summary

Create the directory structure for golden test fixtures. These fixtures are used for conformance testing across all ESRP implementations.

## Context

Test fixtures provide:
- Known-good request/response examples
- Canonical JSON golden files
- Expected hash values
- Cross-language conformance baseline

## Tasks

1. Create fixture directory structure
2. Create README documenting fixture format
3. Set up fixture generation tooling hooks

## Implementation Details

### Directory Structure

Create:

```
fixtures/
└── v1/
    ├── README.md
    ├── requests/
    │   └── .gitkeep
    ├── canonical/
    │   └── .gitkeep
    └── responses/
        └── .gitkeep
```

### fixtures/v1/README.md

```markdown
# ESRP v1 Test Fixtures

This directory contains golden test fixtures for ESRP v1.0 conformance testing.

## Directory Structure

```
v1/
├── requests/          # Input request JSON files
│   ├── simple_tts.json
│   ├── batch_translation.json
│   └── ...
├── canonical/         # Expected canonical JSON output
│   ├── simple_tts.json      # Canonical JSON (no whitespace, sorted keys)
│   ├── simple_tts.sha256    # Expected SHA256 hash
│   └── ...
└── responses/         # Response examples
    ├── simple_tts_success.json
    ├── simple_tts_error.json
    └── ...
```

## Fixture Format

### Request Fixtures

Request fixtures are valid ESRP request JSON files. They should cover:

- Minimal required fields
- All optional fields
- Edge cases (empty arrays, null values)
- Different operation types

### Canonical Fixtures

For each request fixture `foo.json`, there are two canonical files:

- `foo.json` - The request serialized to canonical JSON (sorted keys, no whitespace)
- `foo.sha256` - The SHA256 hash of the canonical JSON (64 hex characters)

### Response Fixtures

Response fixtures show expected response formats:

- `*_success.json` - Successful response
- `*_error.json` - Error response
- `*_async.json` - Async accepted response

## Regenerating Fixtures

Use the ESRP CLI to regenerate canonical fixtures:

```bash
# Generate canonical JSON
esrp canonicalize requests/simple_tts.json > canonical/simple_tts.json

# Generate hash
esrp hash requests/simple_tts.json > canonical/simple_tts.sha256
```

Or use the generation script:

```bash
./tools/generate-fixtures.sh
```

## Adding New Fixtures

1. Create the request JSON in `requests/`
2. Run the generation script to create canonical files
3. If adding a response, create it in `responses/`
4. Commit all files

## Conformance Testing

All ESRP implementations must:

1. Parse all request fixtures without error
2. Produce byte-identical canonical JSON
3. Produce identical SHA256 hashes
4. Pass on all platforms (Linux, macOS, Windows)

## File Naming Convention

- Use `snake_case` for filenames
- Include operation type: `simple_tts`, `batch_translation`
- Suffix with scenario: `_success`, `_error`, `_async`
- Use `.json` extension for JSON, `.sha256` for hashes
```

## Acceptance Criteria

- [ ] Directory structure created
- [ ] README.md explains fixture format
- [ ] `.gitkeep` files ensure directories are tracked
- [ ] No actual fixtures yet (created in subsequent tickets)

## Verification

```bash
ls -la fixtures/v1/
# Should show: README.md, requests/, canonical/, responses/
```

## Notes

- Keep fixtures version-specific (v1/ directory)
- Use `.gitkeep` to track empty directories
- Actual fixtures created in tickets 402-404
