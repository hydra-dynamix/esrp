# Ticket 405: Document Fixture Format

**Phase:** 4 - Test Fixtures
**Priority:** Medium
**Dependencies:** [402 - Create Request Fixtures](402-create-request-fixtures.md), [404 - Create Response Fixtures](404-create-response-fixtures.md)
**Blocked By:** Tickets 402, 404

## Summary

Create comprehensive documentation for the fixture format, including how to add new fixtures, how they're used in testing, and conformance requirements.

## Context

Documentation ensures:
- New contributors can add fixtures correctly
- Cross-language implementers understand the format
- Conformance testing is well-defined
- Fixture regeneration is documented

## Tasks

1. Expand fixtures README with detailed format specification
2. Document conformance testing requirements
3. Create contributing guide for fixtures
4. Document CI integration

## Implementation Details

### fixtures/v1/README.md (Expanded)

Update the README from ticket 401 with:

```markdown
# ESRP v1 Test Fixtures

## Overview

This directory contains golden test fixtures for ESRP v1.0 conformance testing. All ESRP implementations (Rust, Python, TypeScript) must pass these tests.

## Fixture Categories

### Request Fixtures (`requests/`)

| Fixture | Description | Key Features |
|---------|-------------|--------------|
| `simple_tts.json` | Minimal TTS request | Required fields only |
| `batch_translation.json` | Multiple inputs | Array of inputs, metadata |
| `image_generation.json` | Full-featured request | All optional fields |
| `async_video.json` | Async mode | mode.type = "async" |

### Canonical Fixtures (`canonical/`)

For each request `foo.json`:

| File | Content |
|------|---------|
| `foo.json` | Canonical JSON (sorted keys, no whitespace) |
| `foo.sha256` | SHA256 of canonical JSON (64 hex chars) |
| `foo.payload.sha256` | SHA256 of payload (target+inputs+params) |

### Response Fixtures (`responses/`)

| Fixture | Description | Status |
|---------|-------------|--------|
| `simple_tts_success.json` | Successful TTS | succeeded |
| `simple_tts_error.json` | Backend error | failed |
| `validation_error.json` | Invalid input | failed |
| `async_video_accepted.json` | Job queued | accepted |
| `async_video_completed.json` | Job done | succeeded |

## Canonical JSON Specification

Canonical JSON follows these **NORMATIVE** rules:

1. **Object keys**: Sorted lexicographically by UTF-8 bytes
2. **Arrays**: Preserve insertion order (do NOT sort)
3. **Numbers**: Integers only (floats PROHIBITED)
4. **Whitespace**: None (compact format)
5. **Encoding**: UTF-8 only
6. **Booleans**: Lowercase `true`/`false`
7. **Null**: Lowercase `null`

### Example

Input:
```json
{
  "z_key": 1,
  "a_key": 2,
  "nested": {"b": 1, "a": 2}
}
```

Canonical output:
```json
{"a_key":2,"nested":{"a":2,"b":1},"z_key":1}
```

## Conformance Requirements

All implementations MUST:

1. **Parse all fixtures** without error
2. **Produce byte-identical canonical JSON** as golden files
3. **Produce identical SHA256 hashes** as golden files
4. **Pass on all platforms**: Linux, macOS, Windows

### Hash Verification

```
computed_hash = sha256(canonical_json(request))
assert computed_hash == content_of(canonical/foo.sha256)
```

### Payload Hash Verification

```
payload = { target, inputs, params }
computed_hash = sha256(canonical_json(payload))
assert computed_hash == content_of(canonical/foo.payload.sha256)
```

## Adding New Fixtures

### Step 1: Create Request Fixture

Create `requests/new_scenario.json`:

```json
{
  "esrp_version": "1.0",
  "request_id": "...",
  ...
}
```

Requirements:
- Use fixed UUID (not random)
- Use fixed timestamp: `2025-01-01T00:00:00Z`
- No floats in `params` (use strings)
- Pretty-print for readability

### Step 2: Generate Canonical Fixtures

```bash
./tools/generate-fixtures.sh
```

This creates:
- `canonical/new_scenario.json`
- `canonical/new_scenario.sha256`
- `canonical/new_scenario.payload.sha256`

### Step 3: Add Response Fixture (if needed)

Create `responses/new_scenario_success.json` (and error variant).

### Step 4: Update Tests

Add the new fixture to the conformance test list (if not auto-discovered).

### Step 5: Commit

```bash
git add fixtures/v1/
git commit -m "Add new_scenario fixture"
```

## CI Integration

### GitHub Actions

```yaml
- name: Run conformance tests
  run: |
    cargo test --package esrp-core conformance
    pytest tests/conformance/
    npm test -- conformance

- name: Verify fixtures are up-to-date
  run: |
    ./tools/generate-fixtures.sh
    git diff --exit-code fixtures/
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

./tools/generate-fixtures.sh
git diff --exit-code fixtures/v1/canonical/
if [ $? -ne 0 ]; then
    echo "Canonical fixtures are out of date. Run ./tools/generate-fixtures.sh"
    exit 1
fi
```

## Troubleshooting

### Hash Mismatch

1. Check for floats in `params` (use strings)
2. Verify canonical JSON has no whitespace
3. Check key sorting (lexicographic by UTF-8 bytes)
4. Verify UTF-8 encoding (no BOM)

### Platform Differences

1. Use LF line endings (not CRLF)
2. No trailing whitespace
3. No file encoding markers

### Regenerating Fixtures

If the canonical implementation changes:

```bash
# Regenerate all
./tools/generate-fixtures.sh

# Verify tests pass
cargo test conformance

# Commit updates
git add fixtures/
git commit -m "Regenerate fixtures after canonical change"
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-01 | Initial fixture set |
```

## Acceptance Criteria

- [ ] README documents all fixture types
- [ ] Canonical JSON rules clearly specified
- [ ] Conformance requirements documented
- [ ] Adding fixtures guide complete
- [ ] CI integration documented
- [ ] Troubleshooting section included

## Verification

Review the README for completeness:
- Can a new contributor add a fixture?
- Can a Python implementer understand the format?
- Are all edge cases documented?

## Notes

- Keep documentation close to fixtures (in fixtures/v1/)
- Include examples for clarity
- Link to spec for normative rules
- Document common mistakes
