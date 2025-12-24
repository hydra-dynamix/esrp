# Ticket 701: Create esrp-cli Crate

**Phase:** 7 - CLI Tool
**Priority:** Medium
**Dependencies:** Phase 2 and 3 completion
**Blocked By:** esrp-canonical and esrp-workspace

## Summary

Set up the `esrp-cli` binary crate for command-line operations.

## Tasks

1. Update Cargo.toml
2. Create CLI structure with clap
3. Add basic help command

## Implementation Details

### tools/esrp-cli/Cargo.toml

```toml
[package]
name = "esrp-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "esrp"
path = "src/main.rs"

[dependencies]
esrp-core = { path = "../../crates/esrp-core" }
esrp-canonical = { path = "../../crates/esrp-canonical" }
esrp-workspace = { path = "../../crates/esrp-workspace" }
clap = { version = "4.0", features = ["derive"] }
serde_json = { workspace = true }
anyhow = "1.0"
```

### src/main.rs

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "esrp")]
#[command(about = "ESRP Command Line Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate { file: String },
    Canonicalize { file: String },
    Hash { file: String },
    Parse { uri: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // Handle commands...
    Ok(())
}
```

## Acceptance Criteria

- [ ] CLI compiles
- [ ] `esrp --help` shows usage
- [ ] Can be installed with `cargo install`
