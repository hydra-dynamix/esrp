# esrp-py

Python bindings for the Erasmus Service Request Protocol (ESRP).

## Installation

```bash
pip install esrp-py
```

Or build from source:

```bash
cd bindings/python
maturin develop
```

## Usage

```python
import esrp_py

# Validate an ESRP request
request_json = '{"esrp_version": "1.0", ...}'
esrp_py.validate_request(request_json)

# Canonicalize JSON
canonical = esrp_py.canonicalize('{"b": 2, "a": 1}')
# Returns: '{"a":1,"b":2}'

# Compute hash
hash_value = esrp_py.hash_json('{"a": 1}')
# Returns: 64-character hex string

# Parse workspace URI
uri_parts = esrp_py.parse_workspace_uri("workspace://artifacts/output.wav")
# Returns: {"namespace": "artifacts", "path": "output.wav"}

# Check version compatibility
esrp_py.is_version_compatible("1.0")  # True

# Get current version
esrp_py.current_version()  # "1.0"
```

## Functions

- `validate_request(json_str)` - Validate an ESRP request
- `validate_response(json_str)` - Validate an ESRP response
- `canonicalize(json_str)` - Convert to canonical JSON
- `hash_json(json_str)` - Compute SHA256 hash of canonical JSON
- `verify_hash(json_str, expected_hash)` - Verify hash matches
- `parse_workspace_uri(uri)` - Parse workspace URI
- `derive_payload_hash(json_str)` - Derive payload hash for idempotency
- `is_version_compatible(version)` - Check version compatibility
- `current_version()` - Get current ESRP version
