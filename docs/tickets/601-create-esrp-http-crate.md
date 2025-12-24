# Ticket 601: Create esrp-http Crate

**Phase:** 6 - HTTP Transport
**Priority:** High
**Dependencies:** Phase 5 completion
**Blocked By:** Conformance tests passing

## Summary

Set up the `esrp-http` crate for HTTP transport layer with Axum server components and Reqwest client.

## Tasks

1. Update Cargo.toml with dependencies
2. Create module structure
3. Add crate documentation

## Implementation Details

### Cargo.toml

```toml
[package]
name = "esrp-http"
version.workspace = true
edition.workspace = true

[dependencies]
esrp-core = { path = "../esrp-core" }
esrp-canonical = { path = "../esrp-canonical" }
axum = { workspace = true }
reqwest = { workspace = true }
tokio = { workspace = true }
tower-http = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
```

### Module Structure

```
crates/esrp-http/src/
├── lib.rs
├── extractors.rs   # Axum extractors
├── response.rs     # Response helpers
├── client.rs       # Reqwest client
└── error.rs        # HTTP error types
```

## Acceptance Criteria

- [ ] Crate compiles
- [ ] Depends on esrp-core and esrp-canonical
- [ ] Module structure created
