# Ticket 305: Write Workspace Tests

**Phase:** 3 - Workspace Semantics
**Priority:** High
**Dependencies:** [304 - Implement FilesystemWorkspace](304-implement-filesystem-workspace.md)
**Blocked By:** Ticket 304

## Summary

Write comprehensive integration tests for the `esrp-workspace` crate covering edge cases, error handling, and real-world scenarios.

## Context

Workspace tests ensure:
- URI parsing handles all edge cases
- Filesystem operations work correctly
- Error handling is robust
- Cross-platform compatibility (Windows/Unix paths)

## Tasks

1. Create test file structure
2. Write URI parsing edge case tests
3. Write filesystem operation tests
4. Write error handling tests
5. Write cross-platform tests

## Implementation Details

### Test File Structure

```
crates/esrp-workspace/tests/
├── uri_test.rs           # URI parsing tests
├── filesystem_test.rs    # Filesystem provider tests
└── integration_test.rs   # End-to-end tests
```

### tests/integration_test.rs

```rust
//! Integration tests for esrp-workspace

use esrp_workspace::{
    FilesystemWorkspace, StoredArtifact, WorkspaceError, WorkspaceProvider,
    WorkspaceProviderExt, WorkspaceUri,
};
use tempfile::TempDir;

fn temp_workspace() -> (TempDir, FilesystemWorkspace) {
    let dir = TempDir::new().unwrap();
    let workspace = FilesystemWorkspace::new(dir.path());
    (dir, workspace)
}

mod end_to_end {
    use super::*;

    #[test]
    fn test_full_workflow() {
        let (_dir, workspace) = temp_workspace();

        // 1. Store artifact
        let data = b"artifact content";
        let uri = workspace.store("artifacts", data).unwrap();

        // 2. Verify it exists
        assert!(workspace.exists(&uri).unwrap());

        // 3. Get size
        assert_eq!(workspace.size(&uri).unwrap(), 16);

        // 4. Get hash
        let hash = workspace.hash(&uri).unwrap();
        assert_eq!(hash.len(), 64);

        // 5. Verify hash
        assert!(workspace.verify(&uri, &hash).unwrap());

        // 6. Retrieve data
        let retrieved = workspace.retrieve(&uri).unwrap();
        assert_eq!(retrieved, data);

        // 7. Delete
        workspace.delete(&uri).unwrap();
        assert!(!workspace.exists(&uri).unwrap());
    }

    #[test]
    fn test_store_with_metadata() {
        let (_dir, workspace) = temp_workspace();

        let data = b"test data";
        let artifact: StoredArtifact = workspace.store_with_metadata("ns", data).unwrap();

        assert_eq!(artifact.size_bytes, 9);
        assert_eq!(artifact.sha256.len(), 64);
        assert!(workspace.exists(&artifact.uri).unwrap());
    }

    #[test]
    fn test_retrieve_verified() {
        let (_dir, workspace) = temp_workspace();

        let data = b"verified data";
        let artifact = workspace.store_with_metadata("ns", data).unwrap();

        // Correct hash should work
        let retrieved = workspace
            .retrieve_verified(&artifact.uri, &artifact.sha256)
            .unwrap();
        assert_eq!(retrieved, data);

        // Wrong hash should fail
        let wrong_hash = "a".repeat(64);
        let result = workspace.retrieve_verified(&artifact.uri, &wrong_hash);
        assert!(matches!(result, Err(WorkspaceError::HashMismatch { .. })));
    }
}

mod multiple_namespaces {
    use super::*;

    #[test]
    fn test_namespace_isolation() {
        let (_dir, workspace) = temp_workspace();

        // Store same filename in different namespaces
        let uri1 = WorkspaceUri::new("ns1", "file.txt").unwrap();
        let uri2 = WorkspaceUri::new("ns2", "file.txt").unwrap();

        workspace.store_at(&uri1, b"data1").unwrap();
        workspace.store_at(&uri2, b"data2").unwrap();

        // Should retrieve correct data for each
        assert_eq!(workspace.retrieve(&uri1).unwrap(), b"data1");
        assert_eq!(workspace.retrieve(&uri2).unwrap(), b"data2");
    }

    #[test]
    fn test_many_namespaces() {
        let (_dir, workspace) = temp_workspace();

        for i in 0..10 {
            let ns = format!("namespace-{}", i);
            let data = format!("data-{}", i).into_bytes();
            workspace.store(&ns, &data).unwrap();
        }

        // All namespaces should be accessible
        // (verified by no errors during store)
    }
}

mod large_files {
    use super::*;

    #[test]
    fn test_large_file() {
        let (_dir, workspace) = temp_workspace();

        // 1 MB file
        let data = vec![0u8; 1_000_000];
        let uri = workspace.store("large", &data).unwrap();

        let retrieved = workspace.retrieve(&uri).unwrap();
        assert_eq!(retrieved.len(), 1_000_000);
    }

    #[test]
    fn test_empty_file() {
        let (_dir, workspace) = temp_workspace();

        let data = b"";
        let uri = workspace.store("empty", data).unwrap();

        let retrieved = workspace.retrieve(&uri).unwrap();
        assert!(retrieved.is_empty());
        assert_eq!(workspace.size(&uri).unwrap(), 0);
    }
}

mod concurrent_access {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_writes_different_files() {
        let (_dir, workspace) = temp_workspace();
        let workspace = Arc::new(workspace);

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let ws = Arc::clone(&workspace);
                thread::spawn(move || {
                    let data = format!("data-{}", i).into_bytes();
                    ws.store("concurrent", &data).unwrap()
                })
            })
            .collect();

        let uris: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All URIs should be unique
        let unique: std::collections::HashSet<_> = uris.iter().collect();
        assert_eq!(unique.len(), 10);
    }

    #[test]
    fn test_concurrent_reads() {
        let (_dir, workspace) = temp_workspace();
        let workspace = Arc::new(workspace);

        // Store once
        let data = b"shared data";
        let uri = workspace.store("shared", data).unwrap();

        // Read concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let ws = Arc::clone(&workspace);
                let uri = uri.clone();
                thread::spawn(move || ws.retrieve(&uri).unwrap())
            })
            .collect();

        for handle in handles {
            let retrieved = handle.join().unwrap();
            assert_eq!(retrieved, data);
        }
    }
}

mod error_cases {
    use super::*;

    #[test]
    fn test_retrieve_nonexistent() {
        let (_dir, workspace) = temp_workspace();

        let uri = WorkspaceUri::new("ns", "missing.txt").unwrap();
        let result = workspace.retrieve(&uri);

        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
    }

    #[test]
    fn test_size_nonexistent() {
        let (_dir, workspace) = temp_workspace();

        let uri = WorkspaceUri::new("ns", "missing.txt").unwrap();
        let result = workspace.size(&uri);

        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
    }

    #[test]
    fn test_hash_nonexistent() {
        let (_dir, workspace) = temp_workspace();

        let uri = WorkspaceUri::new("ns", "missing.txt").unwrap();
        let result = workspace.hash(&uri);

        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
    }

    #[test]
    fn test_verify_nonexistent() {
        let (_dir, workspace) = temp_workspace();

        let uri = WorkspaceUri::new("ns", "missing.txt").unwrap();
        let result = workspace.verify(&uri, &"a".repeat(64));

        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
    }
}

mod uri_edge_cases {
    use super::*;

    #[test]
    fn test_special_characters_in_path() {
        let (_dir, workspace) = temp_workspace();

        // Paths with special (but valid) characters
        let uri = WorkspaceUri::new("ns", "file-with_special.chars.txt").unwrap();
        workspace.store_at(&uri, b"data").unwrap();

        assert!(workspace.exists(&uri).unwrap());
    }

    #[test]
    fn test_deep_nesting() {
        let (_dir, workspace) = temp_workspace();

        let uri = WorkspaceUri::new("ns", "a/b/c/d/e/f/g/h/i/j/file.txt").unwrap();
        workspace.store_at(&uri, b"deep").unwrap();

        assert!(workspace.exists(&uri).unwrap());
        assert_eq!(workspace.retrieve(&uri).unwrap(), b"deep");
    }

    #[test]
    fn test_unicode_in_data() {
        let (_dir, workspace) = temp_workspace();

        let data = "Hello 世界 🌍".as_bytes();
        let uri = workspace.store("unicode", data).unwrap();

        let retrieved = workspace.retrieve(&uri).unwrap();
        assert_eq!(retrieved, data);
    }
}

mod binary_data {
    use super::*;

    #[test]
    fn test_binary_data_with_nulls() {
        let (_dir, workspace) = temp_workspace();

        let data = vec![0u8, 1, 2, 0, 3, 4, 0, 5];
        let uri = workspace.store("binary", &data).unwrap();

        let retrieved = workspace.retrieve(&uri).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_all_byte_values() {
        let (_dir, workspace) = temp_workspace();

        let data: Vec<u8> = (0..=255).collect();
        let uri = workspace.store("allbytes", &data).unwrap();

        let retrieved = workspace.retrieve(&uri).unwrap();
        assert_eq!(retrieved, data);
    }
}
```

### tests/uri_test.rs

```rust
//! URI parsing edge case tests

use esrp_workspace::{WorkspaceError, WorkspaceUri};

mod security {
    use super::*;

    #[test]
    fn test_path_traversal_variants() {
        // Various path traversal attempts
        let attacks = [
            "workspace://ns/../etc/passwd",
            "workspace://ns/..\\windows\\system32",
            "workspace://ns/subdir/../../etc/passwd",
            "workspace://ns/foo/bar/../../../etc/passwd",
            "workspace://ns/....//....//etc/passwd",
        ];

        for attack in attacks {
            let result = WorkspaceUri::parse(attack);
            assert!(
                result.is_err(),
                "Should reject path traversal: {}",
                attack
            );
        }
    }

    #[test]
    fn test_null_byte_injection() {
        // Null bytes in path could cause truncation in C-based systems
        let result = WorkspaceUri::new("ns", "file\0.txt");
        assert!(result.is_err());
    }
}

mod boundary_cases {
    use super::*;

    #[test]
    fn test_min_valid_uri() {
        let uri = WorkspaceUri::parse("workspace://a/b").unwrap();
        assert_eq!(uri.namespace, "a");
        assert_eq!(uri.path.to_string_lossy(), "b");
    }

    #[test]
    fn test_max_namespace_length() {
        let ns = "a".repeat(64);
        let uri = WorkspaceUri::new(&ns, "file").unwrap();
        assert_eq!(uri.namespace.len(), 64);
    }

    #[test]
    fn test_max_path_length() {
        let path = "a".repeat(1024);
        let uri = WorkspaceUri::new("ns", &path).unwrap();
        assert_eq!(uri.path.to_string_lossy().len(), 1024);
    }

    #[test]
    fn test_exceeds_namespace_length() {
        let ns = "a".repeat(65);
        let result = WorkspaceUri::new(&ns, "file");
        assert!(matches!(result, Err(WorkspaceError::NamespaceTooLong(_))));
    }

    #[test]
    fn test_exceeds_path_length() {
        let path = "a".repeat(1025);
        let result = WorkspaceUri::new("ns", &path);
        assert!(matches!(result, Err(WorkspaceError::PathTooLong(_))));
    }
}

mod character_validation {
    use super::*;

    #[test]
    fn test_valid_namespace_characters() {
        let valid = [
            "lowercase",
            "UPPERCASE",
            "MixedCase",
            "with-dash",
            "with_underscore",
            "with.dot",
            "a123",
        ];

        for ns in valid {
            assert!(
                WorkspaceUri::new(ns, "file").is_ok(),
                "Should accept namespace: {}",
                ns
            );
        }
    }

    #[test]
    fn test_invalid_namespace_characters() {
        let invalid = [
            "with space",
            "with/slash",
            "with:colon",
            "with@symbol",
            "with#hash",
            "with$dollar",
            "with%percent",
        ];

        for ns in invalid {
            assert!(
                WorkspaceUri::new(ns, "file").is_err(),
                "Should reject namespace: {}",
                ns
            );
        }
    }
}
```

## Acceptance Criteria

- [ ] End-to-end workflow tests pass
- [ ] Namespace isolation verified
- [ ] Large file handling works
- [ ] Concurrent access is safe
- [ ] All error cases return appropriate errors
- [ ] Security tests (path traversal, null bytes) pass
- [ ] Boundary cases (min/max lengths) pass
- [ ] Character validation works
- [ ] All tests pass on all platforms

## Verification

```bash
cargo test --package esrp-workspace

# Run with verbose output
cargo test --package esrp-workspace -- --nocapture
```

## Notes

- Use `tempfile` crate for test isolation
- Test concurrent access with threads
- Include security-focused tests
- Test boundary conditions explicitly
- Consider platform-specific tests for Windows vs Unix paths
