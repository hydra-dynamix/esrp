# ESRP Development Guidelines

## Git Workflow

### Branch Structure
- `main` - Production branch
- `release-candidate` - Development/staging branch (branches from main, merges into main)
- `feature/*` - Feature branches (branch from release-candidate)

**Status:** release-candidate branch has been created.

### Commit Convention
Use conventional commits with descriptive messages:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Example: `feat: implement canonical JSON serialization for ESRP requests`

### Development Workflow

1. **Create a feature branch:**
   ```
   dev git branch-create <name>
   ```

2. **Before pushing, run CI locally:**
   ```
   dev ci
   ```

3. **If there are issues, fix them:**
   ```
   dev fix
   ```
   Address any issues not resolved by the fix command manually.

4. **Push changes and finalize the branch:**
   ```
   dev git branch-finalize
   ```

5. **Once you have a release built up, bump the version:**
   ```
   dev version bump <major|minor|patch>
   ```
   Then push to release-candidate.

6. **Create a PR to release-candidate:**
   ```
   dev git pr-release
   ```
   This creates the PR with comments listed as the release and updates the changelog.

## Project Structure

See `docs/tickets/000-index.md` for the full implementation plan organized into phases.

## Current Work

**Active Ticket:** [001 - Create Cargo Workspace](docs/tickets/001-create-cargo-workspace.md)
**Branch:** `001-create-cargo-workspace`
**Status:** Ready for PR

### Tasks:
- [x] Create feature branch
- [x] Create workspace root Cargo.toml
- [x] Create crate directories
- [x] Create supporting directories (bindings, fixtures, tests)
- [x] Verify workspace compiles
- [x] Run CI and finalize branch
- [ ] Create PR to release-candidate
