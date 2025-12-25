# ESRP Development Plan
## From Protocol Spec to Production Implementation

**Version:** 1.0  
**Status:** Active  
**Last Updated:** 2025-12-22

---

## Development Philosophy

This plan follows a **foundation-first** approach:

1. Build the protocol core with zero dependencies
2. Add canonicalization and hashing (critical for Synapsis)
3. Layer on transport (HTTP)
4. Create language bindings
5. Migrate existing services incrementally

Each phase has **clear acceptance criteria** that must pass before moving to the next phase.

---

## Phase 0: Repository Setup

### Objectives
- Create workspace structure
- Set up dependency management
- Configure CI/CD skeleton

### Tasks

#### 0.1: Create Cargo Workspace
**What:**
```bash
cargo new --lib esrp
cd esrp
```

**Structure:**
```
esrp/
├── Cargo.toml (workspace root)
├── crates/
│   ├── esrp-core/
│   ├── esrp-canonical/
│   ├── esrp-workspace/
│   └── esrp-http/
├── bindings/
│   ├── python/
│   └── typescript/
├── fixtures/
│   └── v1/
├── tools/
│   └── esrp-cli/
└── tests/
    └── conformance/
```

**Workspace Cargo.toml:**
```toml
[workspace]
members = [
    "crates/esrp-core",
    "crates/esrp-canonical",
    "crates/esrp-workspace",
    "crates/esrp-http",
    "tools/esrp-cli",
]

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

#### 0.2: Create Git Repository
**What:**
- Initialize git
- Add `.gitignore`
- Set up branch protection (main)
- Tag initial commit as `v0.0.0-scaffold`

**Acceptance Criteria:**
- [ ] Workspace compiles with `cargo build`
- [ ] All crate directories exist
- [ ] Git repository initialized
- [ ] `.gitignore` covers `target/`, `Cargo.lock` (for libraries)

---

## Phase 1: Core Protocol Types

### Objectives
- Define all ESRP types in Rust
- Implement validation logic
- Zero external dependencies (except serde, uuid, chrono)
- No canonicalization, no HTTP, just types

### Dependencies
- Phase 0 complete

### Tasks

#### 1.1: Create `esrp-core` Crate

**File:** `crates/esrp-core/Cargo.toml`
```toml
[package]
name = "esrp-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
```

#### 1.2: Implement Core Types

**File:** `crates/esrp-core/src/types.rs`

**What to implement:**
- `ESRPRequest` struct
- `ESRPResponse` struct
- `Caller`, `Target`, `Mode`, `Context` structs
- `Input`, `Output`, `Artifact` structs
- `Timing`, `Error` structs
- `Status`, `ErrorCode`, `RetentionPolicy`, `ArtifactKind` enums
- `JobState`, `JobEvent`, `JobEventType` types

**Key decisions:**
- Use `#[serde(default)]` for optional fields with defaults
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for truly optional fields
- Use `#[serde(rename_all = "...")]` for consistent casing

**Acceptance Criteria:**
- [ ] All types compile
- [ ] All types derive `Debug, Clone, Serialize, Deserialize, PartialEq`
- [ ] Serde derives work (can serialize/deserialize to JSON)
- [ ] No panics in type construction

#### 1.3: Implement Version Validation

**File:** `crates/esrp-core/src/version.rs`

**What to implement:**
```rust
pub const ESRP_MAJOR_VERSION: u8 = 1;
pub const ESRP_MINOR_VERSION: u8 = 0;

pub struct ESRPVersion {
    pub major: u8,
    pub minor: u8,
}

impl ESRPVersion {
    pub fn parse(s: &str) -> Result<Self, VersionError>;
    pub fn is_compatible_with(&self, other: &Self) -> bool;
}

impl Display for ESRPVersion { ... }
impl FromStr for ESRPVersion { ... }
```

**Acceptance Criteria:**
- [ ] Parses valid versions: `"1.0"`, `"1.5"`, `"2.0"`
- [ ] Rejects invalid versions: `""`, `"abc"`, `"1"`, `"1.0.0"`
- [ ] `1.0` is compatible with `1.5` (same major)
- [ ] `1.0` is NOT compatible with `2.0` (different major)

#### 1.4: Implement Input Validation

**File:** `crates/esrp-core/src/validation.rs`

**What to implement:**
```rust
pub enum ValidationError {
    EmptyInputs,
    InvalidEncoding(String),
    InvalidContentType(String),
    InvalidWorkspaceUri(String),
    VersionMismatch { got: String, expected: String },
}

pub fn validate_request(req: &ESRPRequest) -> Result<(), ValidationError>;
pub fn validate_response(res: &ESRPResponse) -> Result<(), ValidationError>;
```

**Validation rules:**
- Inputs list must not be empty
- Encoding must be one of: `utf-8`, `base64`, `path`
- Content-type must be non-empty
- If artifact uri starts with `workspace://`, validate format
- Version must be compatible with `1.x`

**Acceptance Criteria:**
- [ ] Valid requests pass validation
- [ ] Invalid requests return appropriate error
- [ ] All edge cases covered (empty strings, null fields, etc.)

#### 1.5: Write Unit Tests

**File:** `crates/esrp-core/tests/types_test.rs`

**Test coverage:**
- Serialization/deserialization round-trips
- Default values are set correctly
- Optional fields serialize correctly (omitted when None)
- Validation catches all error cases
- Version parsing edge cases

**Acceptance Criteria:**
- [ ] `cargo test --package esrp-core` passes
- [ ] 100% coverage of validation logic
- [ ] Tests include malformed JSON

---

## Phase 2: Canonical Representation

### Objectives
- Implement deterministic JSON serialization
- Implement SHA256 hashing
- Implement payload hash derivation
- Produce byte-identical output across platforms
- **This MUST come before workspace because artifact identity depends on hashing**

### Dependencies
- Phase 1 complete (types defined)

### Tasks

#### 2.1: Create `esrp-canonical` Crate

**File:** `crates/esrp-canonical/Cargo.toml`
```toml
[package]
name = "esrp-canonical"
version = "0.1.0"
edition = "2021"

[dependencies]
esrp-core = { path = "../esrp-core" }
serde = { workspace = true }
serde_json = { workspace = true }
sha2 = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
hex = "0.4"
```

#### 2.2: Implement Canonical JSON

**File:** `crates/esrp-canonical/src/lib.rs`

**What to implement:**
```rust
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError>;

fn canonical_json_value(value: &serde_json::Value) -> Result<Vec<u8>, CanonicalError>;
```

**Rules (from spec):**
- Object keys sorted lexicographically
- Arrays preserve order
- No whitespace
- UTF-8 encoding
- Integers as decimal, no leading zeros
- **Floats NOT ALLOWED** (return error)

**Acceptance Criteria:**
- [ ] Objects with different key order produce identical output
- [ ] Nested objects recursively sorted
- [ ] Arrays preserve order
- [ ] No whitespace in output
- [ ] Floats in input trigger error

#### 2.3: Implement Hashing

**What to implement:**
```rust
pub fn hash_canonical<T: Serialize>(value: &T) -> Result<String, CanonicalError>;
```

**Implementation:**
- Serialize to canonical JSON
- Compute SHA256 over bytes
- Return hex string (64 characters)

**Acceptance Criteria:**
- [ ] Same input produces same hash (deterministic)
- [ ] Different input produces different hash
- [ ] Hash is valid hex (64 chars, `[0-9a-f]+`)

#### 2.4: Implement Payload Hash Derivation

**What to implement:**
```rust
pub fn derive_payload_hash(
    target: &esrp_core::Target,
    inputs: &[esrp_core::Input],
    params: &serde_json::Value,
) -> Result<String, CanonicalError>;

pub fn derive_idempotency_key(
    target: &esrp_core::Target,
    inputs: &[esrp_core::Input],
    params: &serde_json::Value,
) -> Result<String, CanonicalError>;
```

**Implementation:**
- Construct `{ "target": {...}, "inputs": [...], "params": {...} }`
- Canonicalize
- Hash
- Return hex string

**Acceptance Criteria:**
- [ ] Same inputs/params/target → same hash
- [ ] Different order of object keys → same hash (canonical)
- [ ] Different target.service → different hash
- [ ] `derive_idempotency_key` is alias for `derive_payload_hash`

#### 2.5: Document Number Handling

**File:** `crates/esrp-canonical/README.md`

**What to document:**
- Why floats are banned
- How to represent floats (use strings)
- Examples of valid/invalid inputs
- Cross-platform compatibility guarantees

**Acceptance Criteria:**
- [ ] README explains float restriction
- [ ] Examples show correct usage
- [ ] Rustdoc comments on public functions

#### 2.6: Write Canonical Tests

**File:** `crates/esrp-canonical/tests/canonical_test.rs`

**Test cases:**
- Object key ordering
- Nested objects
- Arrays preserve order
- No whitespace
- Float rejection
- Hash determinism
- Cross-platform hash stability (fixtures)

**Acceptance Criteria:**
- [ ] `cargo test --package esrp-canonical` passes
- [ ] All edge cases covered

---

## Phase 3: Workspace Semantics

### Objectives
- Define workspace URI parser
- Implement `WorkspaceProvider` trait
- Implement filesystem-based workspace
- Handle artifact storage/retrieval

### Dependencies
- Phase 1 complete (types defined)

### Tasks

#### 3.1: Create `esrp-workspace` Crate

**File:** `crates/esrp-workspace/Cargo.toml`
```toml
[package]
name = "esrp-workspace"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = { workspace = true }
sha2 = { workspace = true }
uuid = { workspace = true }

[dev-dependencies]
tempfile = "3.0"
```

#### 3.2: Implement URI Parser

**File:** `crates/esrp-workspace/src/uri.rs`

**What to implement:**
```rust
pub struct WorkspaceUri {
    pub namespace: String,
    pub path: PathBuf,
}

impl WorkspaceUri {
    pub fn parse(uri: &str) -> Result<Self, WorkspaceError>;
    pub fn to_string(&self) -> String;
}

impl Display for WorkspaceUri { ... }
impl FromStr for WorkspaceUri { ... }
```

**Validation rules:**
- Must start with `workspace://`
- Namespace: `[a-zA-Z0-9._-]+`, max 64 chars
- Path: relative, no `..`, no leading `/`, max 1024 chars
- UTF-8 only (or percent-encoded)

**Acceptance Criteria:**
- [ ] Valid URIs parse correctly
- [ ] Invalid URIs return errors (wrong prefix, invalid namespace, etc.)
- [ ] Round-trip: parse → to_string → parse equals original
- [ ] Path traversal attacks blocked (`..` rejected)

#### 3.3: Define `WorkspaceProvider` Trait

**File:** `crates/esrp-workspace/src/provider.rs`

**What to implement:**
```rust
pub trait WorkspaceProvider {
    fn resolve(&self, uri: &WorkspaceUri) -> Result<PathBuf, WorkspaceError>;
    fn store(&self, namespace: &str, data: &[u8]) -> Result<WorkspaceUri, WorkspaceError>;
    fn verify(&self, uri: &WorkspaceUri, expected_sha256: &str) -> Result<bool, WorkspaceError>;
}
```

**Acceptance Criteria:**
- [ ] Trait compiles
- [ ] Trait is object-safe (can use `dyn WorkspaceProvider`)

#### 3.4: Implement `FilesystemWorkspace`

**File:** `crates/esrp-workspace/src/filesystem.rs`

**What to implement:**
```rust
pub struct FilesystemWorkspace {
    base_dir: PathBuf,
}

impl FilesystemWorkspace {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self;
}

impl WorkspaceProvider for FilesystemWorkspace {
    // Implement trait methods
}
```

**Implementation details:**
- `resolve()`: `base_dir / namespace / path`
- `store()`: hash content, create file as `<first16chars>.bin`, return URI
- `verify()`: read file, hash, compare

**Acceptance Criteria:**
- [ ] Can store and retrieve blobs
- [ ] SHA256 verification works
- [ ] Namespace isolation (files don't leak between namespaces)
- [ ] Missing files return error
- [ ] Invalid hashes return error

#### 3.5: Write Workspace Tests

**File:** `crates/esrp-workspace/tests/workspace_test.rs`

**Test cases:**
- URI parsing (valid/invalid)
- Filesystem store/resolve round-trip
- SHA256 verification
- Namespace isolation
- Error handling (missing files, invalid paths)

**Acceptance Criteria:**
- [ ] `cargo test --package esrp-workspace` passes
- [ ] Uses `tempfile` for test isolation

---

## Phase 4: Test Fixtures

### Objectives
- Create golden request/response fixtures
- Create canonical JSON fixtures
- Compute expected hashes
- Document fixture format

### Dependencies
- Phase 2 complete (canonicalization implemented)

### Tasks

#### 4.1: Create Fixture Directory Structure

```
fixtures/v1/
├── requests/
│   ├── simple_tts.json
│   ├── batch_translation.json
│   ├── image_generation.json
│   └── async_video.json
├── canonical/
│   ├── simple_tts.json      # Canonical output
│   ├── simple_tts.sha256     # Expected hash
│   └── ...
└── responses/
    ├── simple_tts_success.json
    ├── simple_tts_error.json
    └── ...
```

#### 4.2: Create Request Fixtures

**File:** `fixtures/v1/requests/simple_tts.json`

**What to include:**
- Minimal valid request (text-to-speech)
- All required fields
- No optional fields (to test defaults)

**Example:**
```json
{
  "esrp_version": "1.0",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-01-01T00:00:00Z",
  "caller": {
    "system": "erasmus"
  },
  "target": {
    "service": "tts",
    "operation": "synthesize"
  },
  "inputs": [
    {
      "name": "text",
      "content_type": "text/plain",
      "data": "Hello world",
      "encoding": "utf-8"
    }
  ],
  "params": {
    "voice": "en-US-Standard-A"
  }
}
```

**More fixtures:**
- `batch_translation.json` - multiple inputs
- `image_generation.json` - with causation_id, scope_id
- `async_video.json` - async mode

**Acceptance Criteria:**
- [ ] All fixtures deserialize successfully
- [ ] All fixtures validate successfully
- [ ] At least 4 request fixtures covering different scenarios

#### 4.3: Generate Canonical Fixtures

**Script:** `tools/generate-fixtures.sh`

**What to implement:**
```bash
#!/bin/bash
for req in fixtures/v1/requests/*.json; do
  base=$(basename "$req" .json)
  cargo run --bin esrp-cli -- canonicalize "$req" \
    > "fixtures/v1/canonical/${base}.json"
  cargo run --bin esrp-cli -- hash "$req" \
    > "fixtures/v1/canonical/${base}.sha256"
done
```

**Acceptance Criteria:**
- [ ] Script runs without errors
- [ ] Canonical JSON files are valid JSON
- [ ] Hash files contain valid hex strings (64 chars)
- [ ] Re-running script produces identical output (idempotent)

#### 4.4: Create Response Fixtures

**File:** `fixtures/v1/responses/simple_tts_success.json`

**What to include:**
- Successful response
- Error response
- Async accepted response

**Acceptance Criteria:**
- [ ] All response fixtures deserialize successfully
- [ ] At least 3 response fixtures (success, error, async)

#### 4.5: Document Fixture Format

**File:** `fixtures/v1/README.md`

**What to document:**
- Purpose of fixtures (conformance testing)
- How to add new fixtures
- How to regenerate canonical/hashes
- How fixtures are used in tests

**Acceptance Criteria:**
- [ ] README explains fixture structure
- [ ] Instructions for adding fixtures
- [ ] Instructions for regenerating

---

## Phase 5: Conformance Tests

### Objectives
- Test parsing of all fixtures
- Test canonical JSON matches golden output
- Test hashes match expected values
- Test cross-language conformance (future)

### Dependencies
- Phase 4 complete (fixtures created)

### Tasks

#### 5.1: Create Conformance Test Suite

**File:** `tests/conformance/mod.rs`

**What to implement:**
```rust
#[test]
fn test_parse_all_request_fixtures() {
    for entry in std::fs::read_dir("fixtures/v1/requests/").unwrap() {
        let path = entry.unwrap().path();
        let json = std::fs::read_to_string(&path).unwrap();
        let req: ESRPRequest = serde_json::from_str(&json).unwrap();
        
        // Validate
        assert!(validate_request(&req).is_ok());
    }
}

#[test]
fn test_canonical_json_matches_golden() {
    for entry in std::fs::read_dir("fixtures/v1/requests/").unwrap() {
        let path = entry.unwrap().path();
        let filename = path.file_stem().unwrap().to_str().unwrap();
        
        let req: ESRPRequest = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        
        let canonical = to_canonical_json(&req).unwrap();
        let golden = std::fs::read_to_string(
            format!("fixtures/v1/canonical/{}.json", filename)
        ).unwrap();
        
        assert_eq!(canonical, golden.as_bytes());
    }
}

#[test]
fn test_hashes_match_expected() {
    // Similar to above, but compare hashes
}
```

**Acceptance Criteria:**
- [ ] All fixtures parse successfully
- [ ] Canonical JSON matches golden files byte-for-byte
- [ ] Computed hashes match expected hashes
- [ ] Tests run in CI

#### 5.2: Add Fixture Validation to CI

**File:** `.github/workflows/conformance.yml`

**What to include:**
- Run conformance tests on every commit
- Fail if fixtures don't match
- Test on multiple platforms (Linux, macOS, Windows)

**Acceptance Criteria:**
- [ ] CI runs conformance tests
- [ ] Tests pass on all platforms
- [ ] Failed tests block merges

---

## Phase 6: HTTP Transport

### Objectives
- Implement Axum-based HTTP handlers
- Implement Reqwest-based client
- Map ESRP errors to HTTP status codes
- Support sync and async endpoints

### Dependencies
- Phase 5 complete (conformance passing)

### Tasks

#### 6.1: Create `esrp-http` Crate

**File:** `crates/esrp-http/Cargo.toml`
```toml
[package]
name = "esrp-http"
version = "0.1.0"
edition = "2021"

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
```

#### 6.2: Implement Axum Extractors

**File:** `crates/esrp-http/src/extractors.rs`

**What to implement:**
```rust
pub struct ESRPRequestExtractor(pub ESRPRequest);

#[axum::async_trait]
impl<S> FromRequest<S> for ESRPRequestExtractor
where
    S: Send + Sync,
{
    type Rejection = ESRPError;
    
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Parse JSON body
        // Validate ESRP request
        // Return wrapped request
    }
}
```

**Acceptance Criteria:**
- [ ] Valid requests extract successfully
- [ ] Invalid JSON returns 400 Bad Request
- [ ] Invalid ESRP schema returns 400 Bad Request with error details
- [ ] Version mismatch returns appropriate error

#### 6.3: Implement Response Helpers

**File:** `crates/esrp-http/src/response.rs`

**What to implement:**
```rust
pub struct ESRPResponseJson(pub ESRPResponse);

impl IntoResponse for ESRPResponseJson {
    fn into_response(self) -> Response {
        // Map status to HTTP status code
        // Serialize response to JSON
        // Set Content-Type header
    }
}

pub fn error_response(
    request_id: Uuid,
    code: ErrorCode,
    message: impl Into<String>,
) -> ESRPResponse {
    // Helper to construct error responses
}
```

**Status code mapping:**
- `Status::Succeeded` → 200 OK
- `Status::Accepted` → 202 Accepted
- `Status::Failed` + `ErrorCode::InvalidInput*` → 400 Bad Request
- `Status::Failed` + `ErrorCode::BackendUnavailable` → 502 Bad Gateway
- `Status::Failed` + `ErrorCode::Timeout` → 408 Request Timeout
- `Status::Failed` + `ErrorCode::Oom` → 507 Insufficient Storage

**Acceptance Criteria:**
- [ ] All status codes mapped correctly
- [ ] Error responses include structured error object
- [ ] Response includes timing information

#### 6.4: Implement Reqwest Client

**File:** `crates/esrp-http/src/client.rs`

**What to implement:**
```rust
pub struct ESRPClient {
    client: reqwest::Client,
    base_url: String,
}

impl ESRPClient {
    pub fn new(base_url: impl Into<String>) -> Self;
    
    pub async fn execute(
        &self,
        request: ESRPRequest,
    ) -> Result<ESRPResponse, ClientError>;
    
    pub async fn execute_async(
        &self,
        request: ESRPRequest,
    ) -> Result<JobRef, ClientError>;
    
    pub async fn get_job_status(
        &self,
        job_id: Uuid,
    ) -> Result<ESRPResponse, ClientError>;
}
```

**Acceptance Criteria:**
- [ ] Client sends valid ESRP requests
- [ ] Client parses ESRP responses
- [ ] Client handles HTTP errors gracefully
- [ ] Client supports timeouts
- [ ] Client can be cloned (Arc internally)

#### 6.5: Write HTTP Integration Tests

**File:** `crates/esrp-http/tests/integration_test.rs`

**What to test:**
- Axum server with mock handler
- Client sends request, receives response
- Error responses parse correctly
- Invalid requests rejected

**Acceptance Criteria:**
- [ ] Integration tests pass
- [ ] Mock server/client round-trip works
- [ ] Tests use fixtures from Phase 4

---

## Phase 7: CLI Tool

### Objectives
- Create command-line tool for working with ESRP
- Validate requests/responses
- Canonicalize JSON
- Compute hashes
- View traces

### Dependencies
- Phase 2 complete (canonical)
- Phase 3 complete (workspace)

### Tasks

#### 7.1: Create `esrp-cli` Binary Crate

**File:** `tools/esrp-cli/Cargo.toml`
```toml
[package]
name = "esrp-cli"
version = "0.1.0"
edition = "2021"

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

#### 7.2: Implement Subcommands

**File:** `tools/esrp-cli/src/main.rs`

**Subcommands:**
```bash
esrp validate <file>           # Validate ESRP JSON
esrp canonicalize <file>       # Output canonical JSON
esrp hash <file>               # Compute payload hash
esrp parse <uri>               # Parse workspace URI
esrp verify <uri> <hash>       # Verify artifact hash
```

**Acceptance Criteria:**
- [ ] All subcommands implemented
- [ ] Help text is clear
- [ ] Errors print to stderr
- [ ] Exit codes: 0 = success, 1 = error
- [ ] Can be installed with `cargo install --path tools/esrp-cli`

#### 7.3: Write CLI Tests

**File:** `tools/esrp-cli/tests/cli_test.rs`

**Use `assert_cmd` crate:**
```rust
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
    
    let golden = std::fs::read_to_string(
        "../../fixtures/v1/canonical/simple_tts.json"
    ).unwrap();
    
    assert_eq!(output.stdout, golden.as_bytes());
}
```

**Acceptance Criteria:**
- [ ] All CLI subcommands have tests
- [ ] Tests use fixtures
- [ ] Tests verify exit codes

---

## Phase 8: Python Bindings

### Objectives
- Create PyO3 bindings for Rust core
- Provide Pydantic models for Python
- Implement Python client
- Implement Python server middleware (FastAPI)

### Dependencies
- Phase 1 complete (types)
- Phase 2 complete (canonical)
- Phase 6 complete (HTTP)

### Tasks

#### 8.1: Create PyO3 Bindings

**File:** `bindings/python/Cargo.toml`
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
serde_json = { workspace = true }
```

**What to implement:**
```python
# After compilation, exposes:
import esrp_py

# Validate request JSON
esrp_py.validate_request(json_str: str) -> None  # Raises ValueError

# Canonicalize
esrp_py.canonicalize(json_str: str) -> str

# Hash
esrp_py.hash_canonical(json_str: str) -> str

# Derive payload hash
esrp_py.derive_payload_hash(
    target: dict,
    inputs: list,
    params: dict
) -> str
```

**Acceptance Criteria:**
- [ ] Bindings compile with `maturin build`
- [ ] Python can import `esrp_py`
- [ ] All functions work as expected
- [ ] Errors raise Python exceptions

#### 8.2: Create Pydantic Models

**File:** `bindings/python/esrp/models.py`

**What to implement:**
```python
from pydantic import BaseModel, Field
from uuid import UUID
from datetime import datetime
from typing import Any, Literal, Optional

class Caller(BaseModel):
    system: str = "erasmus"
    agent_id: Optional[str] = None
    run_id: Optional[str] = None

class Target(BaseModel):
    service: str
    operation: str
    variant: Optional[str] = None

# ... (all ESRP types as Pydantic models)

class ESRPRequest(BaseModel):
    esrp_version: str = "1.0"
    request_id: UUID = Field(default_factory=uuid4)
    # ... (all fields)
    
    def to_canonical(self) -> bytes:
        """Convert to canonical JSON bytes using Rust."""
        import esrp_py
        return esrp_py.canonicalize(self.model_dump_json())
    
    def payload_hash(self) -> str:
        """Compute payload hash using Rust."""
        import esrp_py
        return esrp_py.derive_payload_hash(
            self.target.model_dump(),
            [i.model_dump() for i in self.inputs],
            self.params,
        )
```

**Acceptance Criteria:**
- [ ] All ESRP types have Pydantic models
- [ ] Models validate correctly
- [ ] Models round-trip through JSON
- [ ] Rust validation functions accessible

#### 8.3: Create Python Client

**File:** `bindings/python/esrp/client.py`

**What to implement:**
```python
import httpx
from esrp.models import ESRPRequest, ESRPResponse

class ESRPClient:
    def __init__(self, base_url: str, timeout: float = 30.0):
        self.base_url = base_url.rstrip('/')
        self.client = httpx.Client(timeout=timeout)
    
    def execute(self, request: ESRPRequest) -> ESRPResponse:
        response = self.client.post(
            f"{self.base_url}/v1/execute",
            json=request.model_dump(mode='json'),
        )
        response.raise_for_status()
        return ESRPResponse(**response.json())
    
    # Async version
    # ...
```

**Acceptance Criteria:**
- [ ] Client can send ESRP requests
- [ ] Client can parse ESRP responses
- [ ] Client handles HTTP errors
- [ ] Both sync and async clients work

#### 8.4: Create FastAPI Middleware

**File:** `bindings/python/esrp/fastapi.py`

**What to implement:**
```python
from fastapi import Request, Response
from esrp.models import ESRPRequest, ESRPResponse

async def esrp_middleware(request: Request) -> ESRPRequest:
    """Extract and validate ESRP request from FastAPI request."""
    body = await request.json()
    esrp_req = ESRPRequest(**body)
    
    # Validate using Rust
    import esrp_py
    esrp_py.validate_request(esrp_req.model_dump_json())
    
    return esrp_req

def esrp_response(response: ESRPResponse) -> Response:
    """Convert ESRP response to FastAPI response."""
    status_code = {
        "succeeded": 200,
        "accepted": 202,
        "failed": 500,  # or map based on error code
    }[response.status]
    
    return Response(
        content=response.model_dump_json(),
        media_type="application/json",
        status_code=status_code,
    )
```

**Acceptance Criteria:**
- [ ] Middleware extracts ESRP requests
- [ ] Middleware validates requests
- [ ] Response helper works
- [ ] Integration with FastAPI app works

#### 8.5: Write Python Tests

**File:** `bindings/python/tests/test_conformance.py`

**What to test:**
- Parse all fixtures
- Canonical JSON matches golden
- Hashes match expected
- Client/server round-trip

**Use pytest:**
```python
import pytest
from pathlib import Path
from esrp.models import ESRPRequest
import esrp_py

def test_parse_fixtures():
    fixtures_dir = Path(__file__).parent.parent.parent.parent / "fixtures/v1/requests"
    for fixture in fixtures_dir.glob("*.json"):
        req = ESRPRequest.parse_file(fixture)
        assert req.esrp_version.startswith("1.")

def test_canonical_matches_golden():
    # Similar to Rust tests
    pass
```

**Acceptance Criteria:**
- [ ] `pytest` passes all tests
- [ ] Conformance tests match Rust results

---

## Phase 9: Service Migration

### Objectives
- Update existing Rust HTTP proxies to use ESRP
- Maintain backward compatibility during migration
- Validate ESRP integration works end-to-end

### Dependencies
- Phase 8 complete (bindings available)

### Tasks

#### 9.1: Migrate `translator` Service

**File:** `translator/src/main.rs`

**What to change:**
1. Replace custom request/response types with `ESRPRequest`/`ESRPResponse`
2. Update `POST /translate` to `POST /v1/execute`
3. Extract inputs from ESRP format
4. Return ESRP response format

**Before:**
```rust
#[derive(Deserialize)]
struct TranslateRequest {
    text: String,
    source_lang: String,
    target_lang: String,
}
```

**After:**
```rust
use esrp_http::extractors::ESRPRequestExtractor;
use esrp_http::response::ESRPResponseJson;

async fn execute(
    State(state): State<Arc<AppState>>,
    ESRPRequestExtractor(request): ESRPRequestExtractor,
) -> Result<ESRPResponseJson, StatusCode> {
    // Extract text from inputs[0]
    let text = request.inputs.first()
        .ok_or(StatusCode::BAD_REQUEST)?
        .data.clone();
    
    // Extract params
    let source_lang = request.params.get("source_lang")...
    let target_lang = request.params.get("target_lang")...
    
    // Forward to backend (existing logic)
    // ...
    
    // Return ESRP response
    Ok(ESRPResponseJson(ESRPResponse {
        esrp_version: "1.0".to_string(),
        request_id: request.request_id,
        status: Status::Succeeded,
        outputs: vec![Output {
            name: "translation".to_string(),
            content_type: "text/plain".to_string(),
            data: translation,
            encoding: "utf-8".to_string(),
            metadata: serde_json::json!({}),
        }],
        // ...
    }))
}
```

**Acceptance Criteria:**
- [ ] Service compiles with ESRP types
- [ ] Service accepts ESRP requests
- [ ] Service returns ESRP responses
- [ ] Backward compatibility (old endpoint deprecated but still works)
- [ ] Integration test with Python client works

#### 9.2: Migrate Other Services

Repeat for:
- `text2speech`
- `speech2text`
- `text2video`

**Acceptance Criteria:**
- [ ] All services use ESRP
- [ ] All services pass integration tests
- [ ] Documentation updated

#### 9.3: Update Erasmus Python Services

**File:** `services/generators/text2speech/service.py`

**What to change:**
1. Import `esrp` package
2. Update service to accept ESRP requests (or add ESRP adapter layer)
3. Return ESRP responses

**Acceptance Criteria:**
- [ ] Python services use ESRP models
- [ ] Services can be called via ESRP client
- [ ] Tests updated

---

## Phase 10: Event Logger

### Objectives
- Implement simple append-only event log
- Log all ESRP requests/responses
- Support trace queries
- Prepare for Synapsis integration

### Dependencies
- Phase 5 complete (conformance)
- Phase 9 complete (services migrated)

### Tasks

#### 10.1: Create Event Logger

**File:** `crates/esrp-core/src/event_log.rs`

**What to implement:**
```rust
pub struct EventLog {
    db: SqliteConnection,
}

pub enum EventType {
    ServiceRequested,
    ServiceCompleted,
    ServiceFailed,
    ArtifactCreated,
}

pub struct Event {
    pub id: i64,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
    pub trace_id: Uuid,
    pub service: String,
    pub operation: String,
    pub payload: serde_json::Value,
}

impl EventLog {
    pub fn new(db_path: &Path) -> Result<Self>;
    pub fn log_request(&self, req: &ESRPRequest) -> Result<i64>;
    pub fn log_response(&self, req: &ESRPRequest, res: &ESRPResponse) -> Result<i64>;
    pub fn get_trace(&self, trace_id: Uuid) -> Result<Vec<Event>>;
    pub fn get_request_events(&self, request_id: Uuid) -> Result<Vec<Event>>;
}
```

**Schema:**
```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    request_id TEXT NOT NULL,
    trace_id TEXT,
    scope_id TEXT,
    causation_id TEXT,
    service TEXT,
    operation TEXT,
    status TEXT,
    duration_ms REAL,
    payload TEXT NOT NULL,
    INDEX idx_request_id ON events(request_id),
    INDEX idx_trace_id ON events(trace_id),
    INDEX idx_timestamp ON events(timestamp)
);
```

**Acceptance Criteria:**
- [ ] Can log requests and responses
- [ ] Can query by trace_id
- [ ] Can query by request_id
- [ ] Can query by time range
- [ ] Events are append-only (no updates)

#### 10.2: Integrate with Services

**What to add:**
- Middleware that logs all ESRP requests/responses
- Config option to enable/disable logging
- Log to SQLite file

**Acceptance Criteria:**
- [ ] All service calls logged
- [ ] Can reconstruct causation chains from logs
- [ ] No performance impact when logging disabled

#### 10.3: Create Trace Viewer

**File:** `tools/esrp-cli/src/trace.rs`

**Add subcommand:**
```bash
esrp trace <trace_id>    # Show all events in a trace
esrp request <req_id>    # Show request/response pair
esrp timeline <scope_id> # Show causation timeline
```

**Output format:**
```
Trace: 550e8400-e29b-41d4-a716-446655440000
Duration: 1234ms

┌─────────────────────────────────────────────────
│ 2025-01-01 00:00:00.000 service.requested
│   service: tts
│   operation: synthesize
│   request_id: 550e8400...
│
├─────────────────────────────────────────────────
│ 2025-01-01 00:00:01.234 service.completed
│   status: succeeded
│   duration: 1234ms
│   artifacts: 1
└─────────────────────────────────────────────────
```

**Acceptance Criteria:**
- [ ] Trace viewer works
- [ ] Human-readable output
- [ ] Can export as JSON
- [ ] Can filter by service/operation

---

## Acceptance Criteria Summary

### Phase 0: Repository Setup
- [ ] Workspace compiles with `cargo build`
- [ ] Git repository initialized with proper `.gitignore`
- [ ] All crate directories exist

### Phase 1: Core Protocol Types
- [ ] All types compile and derive required traits
- [ ] Version validation works for valid/invalid inputs
- [ ] Input validation catches all error cases
- [ ] `cargo test --package esrp-core` passes

### Phase 2: Canonical Representation
- [ ] Objects with different key order produce identical output
- [ ] Same input produces same hash (deterministic)
- [ ] Floats in canonical regions trigger error
- [ ] `cargo test --package esrp-canonical` passes

### Phase 3: Workspace Semantics
- [ ] Valid URIs parse correctly, invalid URIs rejected
- [ ] Filesystem workspace can store/retrieve/verify
- [ ] Path traversal attacks blocked
- [ ] `cargo test --package esrp-workspace` passes

### Phase 4: Test Fixtures
- [ ] At least 4 request fixtures covering different scenarios
- [ ] Canonical JSON and hash files generated
- [ ] All fixtures deserialize successfully

### Phase 5: Conformance Tests
- [ ] Canonical JSON matches golden files byte-for-byte
- [ ] Computed hashes match expected hashes
- [ ] Tests run in CI on multiple platforms

### Phase 6: HTTP Transport
- [ ] Valid requests extract successfully
- [ ] Error responses include structured error object
- [ ] Client sends valid ESRP requests
- [ ] Integration tests pass with mock server

### Phase 7: CLI Tool
- [ ] All subcommands implemented with clear help text
- [ ] Can validate, canonicalize, hash ESRP files
- [ ] CLI tests pass

### Phase 8: Python Bindings
- [ ] PyO3 bindings compile and import
- [ ] Pydantic models validate correctly
- [ ] Python client can send/receive ESRP
- [ ] `pytest` passes all conformance tests

### Phase 9: Service Migration
- [ ] All services use ESRP request/response types
- [ ] Integration tests with Python client work
- [ ] Documentation updated

### Phase 10: Event Logger
- [ ] Can log and query events
- [ ] Trace viewer shows human-readable output
- [ ] All service calls logged when enabled

---

## Post-Implementation Validation

Before declaring ESRP v1.0 complete:

1. **Cross-language conformance**: All fixtures produce identical canonical JSON and hashes in Rust, Python, and TypeScript
2. **End-to-end test**: Orchestrator → Service A → Service B → Event Log → Trace Reconstruction
3. **Performance test**: 1000 requests/sec with logging enabled
4. **Documentation**: Complete API docs, tutorial, migration guide
5. **Security review**: Input validation, path traversal, hash collisions
6. **Idempotency test**: Duplicate requests with same idempotency_key are deduplicated

---

## Non-Goals (Deferred to v2.0)

- Streaming inputs/outputs
- Schema registry
- Multi-tenancy
- Compression
- Built-in authentication
- Rate limiting
- TypeScript bindings (can use JSON Schema instead initially)

These are explicitly out of scope for v1.0 to maintain focus.
