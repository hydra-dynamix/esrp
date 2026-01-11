//! CLI integration tests using assert_cmd

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn esrp_cmd() -> Command {
    Command::cargo_bin("esrp").unwrap()
}

mod validate {
    use super::*;

    #[test]
    fn test_validate_valid_request() {
        esrp_cmd()
            .arg("validate")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Valid ESRP request"));
    }

    #[test]
    fn test_validate_all_request_fixtures() {
        let fixtures_dir = std::path::Path::new("../../fixtures/v1/requests");

        for entry in fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
            let path = entry.expect("Failed to read entry").path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                esrp_cmd()
                    .arg("validate")
                    .arg(&path)
                    .assert()
                    .success()
                    .stdout(predicate::str::contains("Valid ESRP request"));
            }
        }
    }

    #[test]
    fn test_validate_valid_response() {
        esrp_cmd()
            .arg("validate")
            .arg("--response")
            .arg("../../fixtures/v1/responses/simple_tts_success.json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Valid ESRP response"));
    }

    #[test]
    fn test_validate_nonexistent_file() {
        esrp_cmd()
            .arg("validate")
            .arg("nonexistent.json")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Failed to read file"));
    }

    #[test]
    fn test_validate_invalid_json() {
        // Create a temp file with invalid JSON
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("esrp_test_invalid.json");
        fs::write(&temp_file, "{ invalid json }").unwrap();

        esrp_cmd()
            .arg("validate")
            .arg(&temp_file)
            .assert()
            .failure();

        fs::remove_file(&temp_file).ok();
    }
}

mod canonicalize {
    use super::*;

    #[test]
    fn test_canonicalize_produces_valid_json() {
        let output = esrp_cmd()
            .arg("canonicalize")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run canonicalize");

        assert!(output.status.success());

        // Verify it's valid JSON by parsing it
        let stdout = String::from_utf8(output.stdout).unwrap();
        let _: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
    }

    #[test]
    fn test_canonicalize_idempotent() {
        // Canonicalizing canonical JSON should produce the same output
        let output1 = esrp_cmd()
            .arg("canonicalize")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run canonicalize");

        // Write first output to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("esrp_test_canonical.json");
        fs::write(&temp_file, &output1.stdout).unwrap();

        // Canonicalize the canonical output
        let output2 = esrp_cmd()
            .arg("canonicalize")
            .arg(&temp_file)
            .output()
            .expect("Failed to run canonicalize");

        fs::remove_file(&temp_file).ok();

        // Output should be identical (idempotent)
        assert_eq!(output1.stdout, output2.stdout);
    }

    #[test]
    fn test_canonicalize_no_whitespace() {
        let output = esrp_cmd()
            .arg("canonicalize")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run canonicalize");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();

        // No newlines except at the very end
        assert!(
            !stdout.contains('\n') || stdout.ends_with('\n') && stdout.matches('\n').count() == 1
        );
        // No pretty-printing indentation
        assert!(!stdout.contains("  "));
    }

    #[test]
    fn test_canonicalize_sorted_keys() {
        let output = esrp_cmd()
            .arg("canonicalize")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run canonicalize");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();

        // "caller" should come before "esrp_version" alphabetically
        let caller_pos = stdout.find("\"caller\"").unwrap();
        let esrp_version_pos = stdout.find("\"esrp_version\"").unwrap();
        assert!(caller_pos < esrp_version_pos);
    }
}

mod hash {
    use super::*;

    #[test]
    fn test_hash_output_format() {
        let output = esrp_cmd()
            .arg("hash")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run hash");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let hash = stdout.trim();

        // SHA256 is 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_deterministic() {
        let output1 = esrp_cmd()
            .arg("hash")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run hash");

        let output2 = esrp_cmd()
            .arg("hash")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run hash");

        assert_eq!(output1.stdout, output2.stdout);
    }

    #[test]
    fn test_different_files_different_hashes() {
        let output1 = esrp_cmd()
            .arg("hash")
            .arg("../../fixtures/v1/requests/simple_tts.json")
            .output()
            .expect("Failed to run hash");

        let output2 = esrp_cmd()
            .arg("hash")
            .arg("../../fixtures/v1/requests/batch_translation.json")
            .output()
            .expect("Failed to run hash");

        assert_ne!(output1.stdout, output2.stdout);
    }
}

mod parse {
    use super::*;

    #[test]
    fn test_parse_simple_uri() {
        esrp_cmd()
            .arg("parse")
            .arg("workspace://artifacts/output.wav")
            .assert()
            .success()
            .stdout(predicate::str::contains("Namespace: artifacts"))
            .stdout(predicate::str::contains("Path: output.wav"));
    }

    #[test]
    fn test_parse_nested_path() {
        esrp_cmd()
            .arg("parse")
            .arg("workspace://temp/session123/file.txt")
            .assert()
            .success()
            .stdout(predicate::str::contains("Namespace: temp"))
            .stdout(predicate::str::contains("Path: session123/file.txt"));
    }

    #[test]
    fn test_parse_invalid_uri() {
        esrp_cmd()
            .arg("parse")
            .arg("invalid://uri")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Failed to parse workspace URI"));
    }

    #[test]
    fn test_parse_path_traversal_rejected() {
        esrp_cmd()
            .arg("parse")
            .arg("workspace://artifacts/../secret")
            .assert()
            .failure();
    }
}

mod help {
    use super::*;

    #[test]
    fn test_help_flag() {
        esrp_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("ESRP Command Line Tool"))
            .stdout(predicate::str::contains("validate"))
            .stdout(predicate::str::contains("canonicalize"))
            .stdout(predicate::str::contains("hash"))
            .stdout(predicate::str::contains("parse"));
    }

    #[test]
    fn test_version_flag() {
        esrp_cmd()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("esrp"));
    }

    #[test]
    fn test_no_args_shows_help() {
        esrp_cmd()
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage"));
    }
}
