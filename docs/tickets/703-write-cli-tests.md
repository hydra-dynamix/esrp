# Ticket 703: Write CLI Tests

**Phase:** 7 - CLI Tool
**Priority:** Medium
**Dependencies:** [702 - Implement CLI Subcommands](702-implement-cli-subcommands.md)
**Blocked By:** Ticket 702

## Summary

Write CLI tests using `assert_cmd` crate.

## Implementation Details

```rust
use assert_cmd::Command;

#[test]
fn test_validate_valid_fixture() {
    Command::cargo_bin("esrp")
        .unwrap()
        .arg("validate")
        .arg("../../fixtures/v1/requests/simple_tts.json")
        .assert()
        .success();
}

#[test]
fn test_canonicalize_matches_golden() {
    let output = Command::cargo_bin("esrp")
        .unwrap()
        .arg("canonicalize")
        .arg("../../fixtures/v1/requests/simple_tts.json")
        .output()
        .unwrap();

    let golden = std::fs::read("../../fixtures/v1/canonical/simple_tts.json").unwrap();
    assert_eq!(output.stdout, golden);
}
```

## Acceptance Criteria

- [ ] All subcommands tested
- [ ] Tests use fixtures
- [ ] Exit codes verified
