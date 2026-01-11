//! ESRP Command Line Tool
//!
//! Provides commands for working with ESRP requests and responses:
//! - validate: Validate ESRP JSON files
//! - canonicalize: Generate canonical JSON representation
//! - hash: Compute SHA256 hash of canonical JSON
//! - parse: Parse and display workspace URIs

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use esrp_canonical::{hash_canonical, to_canonical_json};
use esrp_core::{validate_request, validate_response, ESRPRequest, ESRPResponse};
use esrp_workspace::WorkspaceUri;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "esrp")]
#[command(version)]
#[command(about = "ESRP Command Line Tool - Validate, canonicalize, and hash ESRP data")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an ESRP JSON file
    #[command(about = "Validate an ESRP request or response JSON file")]
    Validate {
        /// Path to the JSON file to validate
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Treat the file as a response (default is request)
        #[arg(long, short)]
        response: bool,
    },

    /// Canonicalize a JSON file
    #[command(about = "Output canonical JSON representation")]
    Canonicalize {
        /// Path to the JSON file to canonicalize
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Compute SHA256 hash of canonical JSON
    #[command(about = "Compute SHA256 hash of canonical JSON")]
    Hash {
        /// Path to the JSON file to hash
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Parse a workspace URI
    #[command(about = "Parse and display workspace URI components")]
    Parse {
        /// The workspace URI to parse (e.g., workspace://artifacts/output.wav)
        #[arg(value_name = "URI")]
        uri: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file, response } => handle_validate(&file, response),
        Commands::Canonicalize { file } => handle_canonicalize(&file),
        Commands::Hash { file } => handle_hash(&file),
        Commands::Parse { uri } => handle_parse(&uri),
    }
}

fn handle_validate(file: &PathBuf, is_response: bool) -> Result<()> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    if is_response {
        let response: ESRPResponse = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse {} as ESRP response", file.display()))?;
        validate_response(&response).with_context(|| "Response validation failed")?;
        println!("Valid ESRP response");
    } else {
        let request: ESRPRequest = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse {} as ESRP request", file.display()))?;
        validate_request(&request).with_context(|| "Request validation failed")?;
        println!("Valid ESRP request");
    }

    Ok(())
}

fn handle_canonicalize(file: &PathBuf) -> Result<()> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let value: serde_json::Value = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse {} as JSON", file.display()))?;

    let canonical =
        to_canonical_json(&value).with_context(|| "Failed to generate canonical JSON")?;

    std::io::stdout()
        .write_all(&canonical)
        .with_context(|| "Failed to write output")?;

    Ok(())
}

fn handle_hash(file: &PathBuf) -> Result<()> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let value: serde_json::Value = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse {} as JSON", file.display()))?;

    let hash = hash_canonical(&value).with_context(|| "Failed to compute hash")?;

    println!("{}", hash);

    Ok(())
}

fn handle_parse(uri: &str) -> Result<()> {
    let parsed = WorkspaceUri::parse(uri).with_context(|| "Failed to parse workspace URI")?;

    println!("Namespace: {}", parsed.namespace);
    println!("Path: {}", parsed.path.display());
    println!("Full URI: {}", parsed);

    Ok(())
}
