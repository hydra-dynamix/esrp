# Ticket 001: Create Cargo Workspace

**Phase:** 0 - Repository Setup
**Priority:** Critical (Blocking)
**Dependencies:** None
**Blocked By:** None

## Summary

Initialize the Rust Cargo workspace with all required crate directories and workspace-level dependency management.

## Context

ESRP is implemented as a Rust workspace with multiple crates. Each crate has a specific responsibility:
- `esrp-core` - Core types and validation
- `esrp-canonical` - Deterministic JSON serialization
- `esrp-workspace` - Workspace URI handling
- `esrp-http` - HTTP transport layer
- `esrp-cli` - Command-line tool

## Tasks

1. Create the workspace root `Cargo.toml` with workspace configuration
2. Create empty crate directories under `crates/`
3. Create bindings directories for future Python/TypeScript support
4. Create fixtures and tools directories
5. Verify workspace compiles

## Implementation Details

### Workspace Cargo.toml

Create `Cargo.toml` at project root:

```toml
[workspace]
resolver = "2"
members = [
    "crates/esrp-core",
    "crates/esrp-canonical",
    "crates/esrp-workspace",
    "crates/esrp-http",
    "tools/esrp-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/your-org/esrp"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.10", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
sha2 = "0.10"
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.12", features = ["json"] }
tower-http = { version = "0.5", features = ["cors"] }
```

### Directory Structure

Create the following directory structure:

```
esrp/
├── Cargo.toml (workspace root)
├── crates/
│   ├── esrp-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── esrp-canonical/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── esrp-workspace/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── esrp-http/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── bindings/
│   ├── python/
│   └── typescript/
├── fixtures/
│   └── v1/
│       ├── requests/
│       ├── canonical/
│       └── responses/
├── tools/
│   └── esrp-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── tests/
    └── conformance/
```

### Minimal Crate Cargo.toml

Each crate needs a minimal `Cargo.toml`. Example for `esrp-core`:

```toml
[package]
name = "esrp-core"
version.workspace = true
edition.workspace = true

[dependencies]
```

### Minimal lib.rs

Each library crate needs a minimal `src/lib.rs`:

```rust
//! ESRP Core Types
//!
//! This crate contains the core types for the Erasmus Service Request Protocol.
```

### Minimal main.rs for CLI

The CLI needs a minimal `src/main.rs`:

```rust
fn main() {
    println!("ESRP CLI");
}
```

## Acceptance Criteria

- [ ] `cargo build --workspace` compiles without errors
- [ ] All 5 crate directories exist with valid Cargo.toml
- [ ] Workspace dependencies are defined at root level
- [ ] Directory structure matches specification
- [ ] Empty `bindings/`, `fixtures/`, and `tests/` directories exist

## Verification

```bash
# From project root
cargo build --workspace
cargo check --workspace
```

Both commands should complete without errors.

## Notes

- Keep each crate's dependencies minimal at this stage
- Use workspace inheritance (`version.workspace = true`) for consistency
- The CLI is under `tools/` rather than `crates/` as it's a binary, not a library
