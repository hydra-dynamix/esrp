"""Tests for esrp_py Python bindings.

Run with: pytest tests/test_esrp_py.py

Requires: maturin develop (to install the extension module)
"""

import pytest
import json

# Import the extension module
import esrp_py


class TestValidation:
    """Tests for validation functions."""

    def test_validate_request_valid(self):
        """Valid request should not raise."""
        request = {
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "test"},
            "target": {"service": "test", "operation": "test"},
            "inputs": [
                {
                    "name": "text",
                    "content_type": "text/plain",
                    "data": "hello",
                    "encoding": "utf-8",
                }
            ],
            "params": {},
        }
        esrp_py.validate_request(json.dumps(request))

    def test_validate_request_invalid_json(self):
        """Invalid JSON should raise ValueError."""
        with pytest.raises(ValueError, match="Invalid JSON"):
            esrp_py.validate_request("{ invalid json }")

    def test_validate_request_empty_service(self):
        """Empty service name should raise ValueError."""
        request = {
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "test"},
            "target": {"service": "", "operation": "test"},
            "inputs": [
                {
                    "name": "text",
                    "content_type": "text/plain",
                    "data": "hello",
                    "encoding": "utf-8",
                }
            ],
            "params": {},
        }
        with pytest.raises(ValueError, match="Validation error"):
            esrp_py.validate_request(json.dumps(request))

    def test_validate_response_valid(self):
        """Valid response should not raise."""
        response = {
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "status": "succeeded",
        }
        esrp_py.validate_response(json.dumps(response))


class TestCanonicalization:
    """Tests for canonicalization functions."""

    def test_canonicalize_sorts_keys(self):
        """Keys should be sorted alphabetically."""
        result = esrp_py.canonicalize('{"b": 2, "a": 1}')
        assert result == '{"a":1,"b":2}'

    def test_canonicalize_no_whitespace(self):
        """Output should have no whitespace."""
        result = esrp_py.canonicalize('{"a": 1, "b": 2}')
        assert " " not in result
        assert "\n" not in result

    def test_canonicalize_nested_sorting(self):
        """Nested objects should also be sorted."""
        result = esrp_py.canonicalize('{"z": {"b": 2, "a": 1}, "a": 1}')
        assert result == '{"a":1,"z":{"a":1,"b":2}}'

    def test_canonicalize_float_rejected(self):
        """Floating-point numbers should be rejected."""
        with pytest.raises(ValueError, match="Canonicalization error"):
            esrp_py.canonicalize('{"a": 1.5}')

    def test_canonicalize_integer_accepted(self):
        """Integers should be accepted."""
        result = esrp_py.canonicalize('{"a": 42}')
        assert result == '{"a":42}'


class TestHashing:
    """Tests for hashing functions."""

    def test_hash_json_format(self):
        """Hash should be 64 hex characters."""
        result = esrp_py.hash_json('{"a": 1}')
        assert len(result) == 64
        assert all(c in "0123456789abcdef" for c in result)

    def test_hash_json_deterministic(self):
        """Same input should produce same hash."""
        hash1 = esrp_py.hash_json('{"a": 1}')
        hash2 = esrp_py.hash_json('{"a": 1}')
        assert hash1 == hash2

    def test_hash_json_key_order_independent(self):
        """Key order should not affect hash."""
        hash1 = esrp_py.hash_json('{"a": 1, "b": 2}')
        hash2 = esrp_py.hash_json('{"b": 2, "a": 1}')
        assert hash1 == hash2

    def test_verify_hash_correct(self):
        """Correct hash should return True."""
        json_str = '{"a": 1}'
        hash_value = esrp_py.hash_json(json_str)
        assert esrp_py.verify_hash(json_str, hash_value) is True

    def test_verify_hash_incorrect(self):
        """Incorrect hash should return False."""
        json_str = '{"a": 1}'
        wrong_hash = "0" * 64
        assert esrp_py.verify_hash(json_str, wrong_hash) is False

    def test_verify_hash_case_insensitive(self):
        """Hash comparison should be case-insensitive."""
        json_str = '{"a": 1}'
        hash_value = esrp_py.hash_json(json_str)
        assert esrp_py.verify_hash(json_str, hash_value.upper()) is True


class TestWorkspaceUri:
    """Tests for workspace URI parsing."""

    def test_parse_simple_uri(self):
        """Parse a simple workspace URI."""
        result = esrp_py.parse_workspace_uri("workspace://artifacts/output.wav")
        assert result["namespace"] == "artifacts"
        assert result["path"] == "output.wav"

    def test_parse_nested_path(self):
        """Parse URI with nested path."""
        result = esrp_py.parse_workspace_uri("workspace://temp/session/file.txt")
        assert result["namespace"] == "temp"
        assert result["path"] == "session/file.txt"

    def test_parse_invalid_prefix(self):
        """Invalid prefix should raise ValueError."""
        with pytest.raises(ValueError, match="Invalid workspace URI"):
            esrp_py.parse_workspace_uri("file://path/to/file")

    def test_parse_path_traversal_rejected(self):
        """Path traversal should be rejected."""
        with pytest.raises(ValueError, match="Invalid workspace URI"):
            esrp_py.parse_workspace_uri("workspace://artifacts/../secret")


class TestVersion:
    """Tests for version functions."""

    def test_current_version(self):
        """Current version should be 1.0."""
        assert esrp_py.current_version() == "1.0"

    def test_is_version_compatible_same(self):
        """Same version should be compatible."""
        assert esrp_py.is_version_compatible("1.0") is True

    def test_is_version_compatible_minor_higher(self):
        """Higher minor version should be compatible."""
        assert esrp_py.is_version_compatible("1.1") is True

    def test_is_version_compatible_major_different(self):
        """Different major version should not be compatible."""
        assert esrp_py.is_version_compatible("2.0") is False

    def test_is_version_compatible_invalid(self):
        """Invalid version string should raise ValueError."""
        with pytest.raises(ValueError, match="Invalid version"):
            esrp_py.is_version_compatible("invalid")


class TestPayloadHash:
    """Tests for payload hash derivation."""

    def test_derive_payload_hash(self):
        """Derive payload hash for a request."""
        request = {
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "test"},
            "target": {"service": "tts", "operation": "synthesize"},
            "inputs": [
                {
                    "name": "text",
                    "content_type": "text/plain",
                    "data": "hello",
                    "encoding": "utf-8",
                }
            ],
            "params": {"voice": "en-US"},
        }
        result = esrp_py.derive_payload_hash(json.dumps(request))
        assert len(result) == 64
        assert all(c in "0123456789abcdef" for c in result)

    def test_derive_payload_hash_deterministic(self):
        """Same request should produce same hash."""
        request = {
            "esrp_version": "1.0",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2025-01-01T00:00:00Z",
            "caller": {"system": "test"},
            "target": {"service": "tts", "operation": "synthesize"},
            "inputs": [
                {
                    "name": "text",
                    "content_type": "text/plain",
                    "data": "hello",
                    "encoding": "utf-8",
                }
            ],
            "params": {},
        }
        hash1 = esrp_py.derive_payload_hash(json.dumps(request))
        hash2 = esrp_py.derive_payload_hash(json.dumps(request))
        assert hash1 == hash2
