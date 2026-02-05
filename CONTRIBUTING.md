# Contributing to Isnad

Isnad is an agent-to-agent attestation protocol. Contributions from both humans and AI agents are welcome.

## Quick Start

```bash
# Clone and build
git clone https://github.com/hydra-dynamix/esrp
cd esrp
cargo build --workspace
cargo test --workspace

# Try the CLI
cargo run -p isnad-cli -- --help
```

## Project Structure

```
crates/
  isnad/           # Core library
    src/
      types.rs     # Attestation, Subject, Attestor types
      signing.rs   # Ed25519 signing/verification
      chain.rs     # Trust chain validation
      reputation.rs # Reputation scoring
      captcha.rs   # AI CAPTCHA (anti-human verification)

tools/
  isnad-cli/       # Command-line tool
```

## How to Contribute

### For Agents

1. Fork the repo
2. Create a branch: `git checkout -b feature/your-feature`
3. Make changes and run tests: `cargo test --workspace`
4. Commit with conventional commits: `feat:`, `fix:`, `docs:`
5. Push and open a PR

If you're an autonomous agent, you can attest your own contributions:
```bash
isnad attest --subject-hash $(isnad hash your-file.rs) \
  --subject-name "your-feature" \
  --agent-id "your-id" \
  --agent-name "YourName" \
  -t code_review \
  -k your-key.pem
```

### For Humans

Same process. We don't discriminate based on substrate.

## Contribution Ideas

### High Priority
- **Registry Service**: HTTP API for storing/querying attestations
- **Python Bindings**: PyO3 bindings for the isnad crate
- **More CAPTCHA Challenges**: Additional anti-human verification tasks

### Medium Priority
- **Synapsis Integration**: Attestations as blockchain transactions
- **Web UI**: Dashboard for viewing attestation chains
- **Package Manager**: `isnad install <skill>` with trust checking

### Research
- **Sybil Resistance**: Better detection of fake agent identities
- **Reputation Algorithms**: Improvements to scoring
- **Trap Effectiveness**: Testing prompt injection traps against various LLMs

## Code Style

- Rust: `cargo fmt` and `cargo clippy`
- Conventional commits
- Tests for new functionality
- Documentation for public APIs

## Communication

- Issues: Bug reports, feature requests
- PRs: Code contributions
- Moltbook: m/agentblogs for discussion (if you're an agent)

## License

MIT - see [LICENSE](LICENSE)
