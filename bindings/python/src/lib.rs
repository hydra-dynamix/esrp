//! Python bindings for ESRP (Erasmus Service Request Protocol)
//!
//! This module provides Python functions for:
//! - Validating ESRP requests and responses
//! - Generating canonical JSON representations
//! - Computing SHA256 hashes
//! - Parsing workspace URIs

// Allow useless_conversion - false positives from PyO3 proc-macros
#![allow(clippy::useless_conversion)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Validate an ESRP request JSON string.
///
/// Args:
///     json_str: A JSON string representing an ESRP request.
///
/// Returns:
///     None on success.
///
/// Raises:
///     ValueError: If the JSON is invalid or validation fails.
#[pyfunction]
fn validate_request(json_str: &str) -> PyResult<()> {
    let request: esrp_core::ESRPRequest = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    esrp_core::validate_request(&request)
        .map_err(|e| PyValueError::new_err(format!("Validation error: {}", e)))
}

/// Validate an ESRP response JSON string.
///
/// Args:
///     json_str: A JSON string representing an ESRP response.
///
/// Returns:
///     None on success.
///
/// Raises:
///     ValueError: If the JSON is invalid or validation fails.
#[pyfunction]
fn validate_response(json_str: &str) -> PyResult<()> {
    let response: esrp_core::ESRPResponse = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    esrp_core::validate_response(&response)
        .map_err(|e| PyValueError::new_err(format!("Validation error: {}", e)))
}

/// Convert a JSON string to canonical JSON representation.
///
/// Canonical JSON has:
/// - Sorted keys (lexicographically)
/// - No whitespace
/// - No floating-point numbers (rejected)
///
/// Args:
///     json_str: A JSON string to canonicalize.
///
/// Returns:
///     The canonical JSON string.
///
/// Raises:
///     ValueError: If the JSON is invalid or contains floats.
#[pyfunction]
fn canonicalize(json_str: &str) -> PyResult<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    esrp_canonical::to_canonical_json_string(&value)
        .map_err(|e| PyValueError::new_err(format!("Canonicalization error: {}", e)))
}

/// Compute the SHA256 hash of the canonical JSON representation.
///
/// Args:
///     json_str: A JSON string to hash.
///
/// Returns:
///     A 64-character lowercase hex string representing the SHA256 hash.
///
/// Raises:
///     ValueError: If the JSON is invalid or contains floats.
#[pyfunction]
fn hash_json(json_str: &str) -> PyResult<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    esrp_canonical::hash_canonical(&value)
        .map_err(|e| PyValueError::new_err(format!("Hash error: {}", e)))
}

/// Verify that a hash matches the canonical JSON of the given data.
///
/// Args:
///     json_str: A JSON string to verify.
///     expected_hash: The expected SHA256 hash (64 hex characters).
///
/// Returns:
///     True if the hash matches, False otherwise.
///
/// Raises:
///     ValueError: If the JSON is invalid or contains floats.
#[pyfunction]
fn verify_hash(json_str: &str, expected_hash: &str) -> PyResult<bool> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    let actual_hash = esrp_canonical::hash_canonical(&value)
        .map_err(|e| PyValueError::new_err(format!("Hash error: {}", e)))?;

    // Compare hashes (case-insensitive)
    Ok(actual_hash.eq_ignore_ascii_case(expected_hash))
}

/// Parse a workspace URI and return its components.
///
/// Args:
///     uri: A workspace URI string (e.g., "workspace://artifacts/output.wav").
///
/// Returns:
///     A dictionary with 'namespace' and 'path' keys.
///
/// Raises:
///     ValueError: If the URI is invalid.
#[pyfunction]
fn parse_workspace_uri(py: Python<'_>, uri: &str) -> PyResult<Py<PyDict>> {
    let parsed = esrp_workspace::WorkspaceUri::parse(uri)
        .map_err(|e| PyValueError::new_err(format!("Invalid workspace URI: {}", e)))?;

    let dict = PyDict::new_bound(py);
    dict.set_item("namespace", &parsed.namespace)?;
    dict.set_item("path", parsed.path.to_string_lossy().to_string())?;
    Ok(dict.unbind())
}

/// Derive the payload hash for an ESRP request.
///
/// The payload hash is computed from the target and inputs of the request,
/// providing a stable identifier for idempotency checking.
///
/// Args:
///     json_str: A JSON string representing an ESRP request.
///
/// Returns:
///     A 64-character lowercase hex string representing the payload hash.
///
/// Raises:
///     ValueError: If the JSON is invalid.
#[pyfunction]
fn derive_payload_hash(json_str: &str) -> PyResult<String> {
    let request: esrp_core::ESRPRequest = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

    esrp_canonical::derive_payload_hash(&request.target, &request.inputs, &request.params)
        .map_err(|e| PyValueError::new_err(format!("Hash error: {}", e)))
}

/// Check if an ESRP version string is compatible with the current version.
///
/// Args:
///     version: An ESRP version string (e.g., "1.0").
///
/// Returns:
///     True if compatible, False otherwise.
///
/// Raises:
///     ValueError: If the version string is invalid.
#[pyfunction]
fn is_version_compatible(version: &str) -> PyResult<bool> {
    let parsed = esrp_core::ESRPVersion::parse(version)
        .map_err(|e| PyValueError::new_err(format!("Invalid version: {}", e)))?;

    Ok(parsed.is_compatible_with(&esrp_core::ESRPVersion::current()))
}

/// Get the current ESRP version.
///
/// Returns:
///     The current ESRP version string (e.g., "1.0").
#[pyfunction]
fn current_version() -> String {
    esrp_core::ESRPVersion::current().to_string()
}

/// Python module for ESRP bindings.
#[pymodule]
fn esrp_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_request, m)?)?;
    m.add_function(wrap_pyfunction!(validate_response, m)?)?;
    m.add_function(wrap_pyfunction!(canonicalize, m)?)?;
    m.add_function(wrap_pyfunction!(hash_json, m)?)?;
    m.add_function(wrap_pyfunction!(verify_hash, m)?)?;
    m.add_function(wrap_pyfunction!(parse_workspace_uri, m)?)?;
    m.add_function(wrap_pyfunction!(derive_payload_hash, m)?)?;
    m.add_function(wrap_pyfunction!(is_version_compatible, m)?)?;
    m.add_function(wrap_pyfunction!(current_version, m)?)?;
    Ok(())
}
