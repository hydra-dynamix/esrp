# Ticket 002: Initialize Git Repository

**Phase:** 0 - Repository Setup
**Priority:** Critical (Blocking)
**Dependencies:** [001 - Create Cargo Workspace](001-create-cargo-workspace.md)
**Blocked By:** None

## Summary

Set up Git repository with appropriate `.gitignore`, branch protection, and initial commit tagging.

## Context

The ESRP project needs proper version control setup before implementation begins. This includes ignoring build artifacts, setting up branch naming conventions, and creating an initial tagged commit.

## Tasks

1. Initialize git repository (if not already done)
2. Create comprehensive `.gitignore`
3. Create initial commit with scaffold
4. Tag initial commit as `v0.0.0-scaffold`

## Implementation Details

### .gitignore

Create `.gitignore` at project root:

```gitignore
# Rust build artifacts
/target/
**/target/

# Cargo.lock for libraries (keep for binaries)
# We keep it since we have a CLI binary
# Cargo.lock

# IDE files
.idea/
.vscode/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db

# Python artifacts (for bindings)
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
.venv/
venv/
ENV/

# Node/TypeScript artifacts (for bindings)
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
*.tsbuildinfo

# Test artifacts
*.profraw
*.profdata
coverage/
.coverage
htmlcov/

# Environment files
.env
.env.local
.env.*.local

# Temporary files
tmp/
temp/
*.tmp
*.bak

# Generated documentation
/docs/api/

# Maturin (PyO3) build artifacts
bindings/python/target/
*.whl
```

### Initial Commit

After creating the workspace structure:

```bash
git add .
git commit -m "Initial scaffold: Cargo workspace with crate structure

- Set up workspace with 4 library crates and 1 binary
- Configure workspace-level dependencies
- Create directory structure for bindings, fixtures, and tests
- Add comprehensive .gitignore"

git tag v0.0.0-scaffold
```

### Branch Protection (Optional - GitHub)

If using GitHub, configure branch protection for `main`:
- Require pull request reviews
- Require status checks to pass
- Require branches to be up to date

## Acceptance Criteria

- [ ] Git repository initialized
- [ ] `.gitignore` covers Rust, Python, TypeScript, and IDE artifacts
- [ ] Initial commit created with all scaffold files
- [ ] Tag `v0.0.0-scaffold` created
- [ ] `target/` directory is ignored

## Verification

```bash
# Check git status shows clean working tree
git status

# Check tag exists
git tag -l "v0.0.0-scaffold"

# Verify target is ignored (after a build)
cargo build --workspace
git status  # Should still show clean (target ignored)
```

## Notes

- Keep Cargo.lock in version control since we have a binary crate (esrp-cli)
- The `.gitignore` is comprehensive to support future Python and TypeScript bindings
- Branch protection is optional but recommended for team environments
