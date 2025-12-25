# Ticket 502: Add CI Pipeline

**Phase:** 5 - Conformance Tests
**Priority:** High
**Dependencies:** [501 - Create Conformance Test Suite](501-create-conformance-test-suite.md)
**Blocked By:** Ticket 501

## Summary

Set up GitHub Actions CI pipeline to run conformance tests on all platforms.

## Tasks

1. Create GitHub Actions workflow
2. Configure multi-platform testing
3. Add fixture validation check
4. Configure branch protection

## Implementation Details

### .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build
        run: cargo build --workspace

      - name: Run tests
        run: cargo test --workspace

      - name: Run conformance tests
        run: cargo test conformance

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

  fixtures:
    name: Verify Fixtures
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable

      - name: Build CLI
        run: cargo build --release --bin esrp

      - name: Regenerate fixtures
        run: ./tools/generate-fixtures.sh

      - name: Check for changes
        run: |
          if ! git diff --exit-code fixtures/; then
            echo "Fixtures are out of date. Run ./tools/generate-fixtures.sh"
            exit 1
          fi
```

## Acceptance Criteria

- [ ] CI runs on push and PR
- [ ] Tests run on Linux, macOS, Windows
- [ ] Linting checks pass
- [ ] Fixture validation runs
- [ ] Failed tests block merge

## Verification

Push to a branch and verify CI runs successfully.
