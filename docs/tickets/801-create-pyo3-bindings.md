# Ticket 801: Create PyO3 Bindings

**Phase:** 8 - Python Bindings
**Priority:** Medium
**Dependencies:** Phase 6 completion
**Blocked By:** HTTP transport complete

## Summary

Create PyO3 bindings for Rust core functionality.

## Tasks

1. Set up maturin project in bindings/python
2. Expose validation functions
3. Expose canonicalization functions
4. Expose hashing functions

## Implementation Details

### bindings/python/Cargo.toml

```toml
[package]
name = "esrp-py"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
esrp-core = { path = "../../crates/esrp-core" }
esrp-canonical = { path = "../../crates/esrp-canonical" }
pyo3 = { version = "0.20", features = ["extension-module"] }
serde_json = "1.0"
```

### src/lib.rs

```rust
use pyo3::prelude::*;

#[pyfunction]
fn validate_request(json_str: &str) -> PyResult<()> {
    let request: esrp_core::ESRPRequest = serde_json::from_str(json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    esrp_core::validate_request(&request)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
fn canonicalize(json_str: &str) -> PyResult<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    let canonical = esrp_canonical::to_canonical_json_string(&value)?;
    Ok(canonical)
}

#[pymodule]
fn esrp_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_request, m)?)?;
    m.add_function(wrap_pyfunction!(canonicalize, m)?)?;
    Ok(())
}
```

## Acceptance Criteria

- [ ] `maturin build` succeeds
- [ ] Python can `import esrp_py`
- [ ] Functions work correctly
- [ ] Errors raise Python exceptions
