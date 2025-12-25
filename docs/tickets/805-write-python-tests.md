# Ticket 805: Write Python Tests

**Phase:** 8 - Python Bindings
**Priority:** Medium
**Dependencies:** [804 - Create FastAPI Middleware](804-create-fastapi-middleware.md)
**Blocked By:** Ticket 804

## Summary

Write Python conformance tests matching Rust fixtures.

## Implementation Details

### bindings/python/tests/test_conformance.py

```python
import pytest
from pathlib import Path
from esrp.models import ESRPRequest
import esrp_py

FIXTURES_DIR = Path(__file__).parent.parent.parent.parent / "fixtures/v1"

def test_parse_all_fixtures():
    for fixture in (FIXTURES_DIR / "requests").glob("*.json"):
        req = ESRPRequest.model_validate_json(fixture.read_text())
        assert req.esrp_version.startswith("1.")

def test_canonical_matches_golden():
    for fixture in (FIXTURES_DIR / "requests").glob("*.json"):
        canonical = esrp_py.canonicalize(fixture.read_text())
        golden_path = FIXTURES_DIR / "canonical" / f"{fixture.stem}.json"
        golden = golden_path.read_text()
        assert canonical == golden

def test_hash_matches_golden():
    for fixture in (FIXTURES_DIR / "requests").glob("*.json"):
        hash_result = esrp_py.hash_canonical(fixture.read_text())
        golden_path = FIXTURES_DIR / "canonical" / f"{fixture.stem}.sha256"
        expected = golden_path.read_text().strip()
        assert hash_result == expected
```

## Acceptance Criteria

- [ ] All fixtures parse
- [ ] Canonical JSON matches Rust output
- [ ] Hashes match Rust output
- [ ] pytest passes
