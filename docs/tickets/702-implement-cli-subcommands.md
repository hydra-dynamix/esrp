# Ticket 702: Implement CLI Subcommands

**Phase:** 7 - CLI Tool
**Priority:** Medium
**Dependencies:** [701 - Create esrp-cli Crate](701-create-esrp-cli-crate.md)
**Blocked By:** Ticket 701

## Summary

Implement all CLI subcommands: validate, canonicalize, hash, parse.

## Tasks

1. Implement `validate` - validate ESRP JSON
2. Implement `canonicalize` - output canonical JSON
3. Implement `hash` - compute SHA256 hash
4. Implement `parse` - parse workspace URI

## Implementation Details

```rust
fn handle_validate(file: &str) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(file)?;
    let request: ESRPRequest = serde_json::from_str(&json)?;
    validate_request(&request)?;
    println!("Valid ESRP request");
    Ok(())
}

fn handle_canonicalize(file: &str) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(file)?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    let canonical = to_canonical_json(&value)?;
    std::io::stdout().write_all(&canonical)?;
    Ok(())
}

fn handle_hash(file: &str) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(file)?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    let hash = hash_canonical(&value)?;
    println!("{}", hash);
    Ok(())
}
```

## Acceptance Criteria

- [ ] `esrp validate file.json` works
- [ ] `esrp canonicalize file.json` outputs canonical JSON
- [ ] `esrp hash file.json` outputs 64-char hex hash
- [ ] Exit code 0 on success, 1 on error
