//! Generate canonical fixtures for ESRP conformance testing

use esrp_canonical::{hash_canonical, to_canonical_json_string};
use esrp_core::ESRPRequest;
use std::fs;
use std::path::Path;

fn main() {
    let fixtures_dir = Path::new("fixtures/v1");
    let requests_dir = fixtures_dir.join("requests");
    let canonical_dir = fixtures_dir.join("canonical");

    println!("Generating canonical fixtures...");
    println!();

    let mut count = 0;

    for entry in fs::read_dir(&requests_dir).expect("Failed to read requests directory") {
        let path = entry.expect("Failed to read entry").path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let json =
                fs::read_to_string(&path).expect(&format!("Failed to read {}.json", filename));

            let request: ESRPRequest =
                serde_json::from_str(&json).expect(&format!("Failed to parse {}.json", filename));

            // Generate and write canonical JSON
            let canonical = to_canonical_json_string(&request)
                .expect(&format!("Failed to canonicalize {}.json", filename));
            let canonical_path = canonical_dir.join(format!("{}.json", filename));
            fs::write(&canonical_path, &canonical)
                .expect(&format!("Failed to write {}.json", filename));
            println!("  Generated: canonical/{}.json", filename);

            // Generate and write hash
            let hash =
                hash_canonical(&request).expect(&format!("Failed to hash {}.json", filename));
            let hash_path = canonical_dir.join(format!("{}.sha256", filename));
            fs::write(&hash_path, &hash).expect(&format!("Failed to write {}.sha256", filename));
            println!("  Generated: canonical/{}.sha256", filename);

            count += 1;
        }
    }

    println!();
    println!("Done! Generated {} canonical fixtures.", count);
}
