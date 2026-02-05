//! Isnad CLI - Agent trust attestation tool.
//!
//! # Commands
//!
//! - `isnad keygen` - Generate a new Ed25519 keypair
//! - `isnad hash <file>` - Compute SHA256 hash of a file
//! - `isnad attest` - Create a signed attestation
//! - `isnad verify <file>` - Verify an attestation signature
//! - `isnad trust <hash>` - Check trust status for a subject

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use isnad::{
    Attestation, AttestationType, Attestor, ChainConfig, ChainValidator, Evidence, KeyPair,
    MemoryStore, PublicKey, Subject, SubjectType, TrustAnchor,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "isnad")]
#[command(about = "Agent trust attestation tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Ed25519 keypair
    Keygen {
        /// Output file for secret key (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Also output public key to this file
        #[arg(short, long)]
        public: Option<PathBuf>,
    },

    /// Compute SHA256 hash of a file (for use as content_hash)
    Hash {
        /// File to hash (use - for stdin)
        file: PathBuf,
    },

    /// Create a signed attestation
    Attest {
        /// Subject content hash (use `isnad hash` to compute)
        #[arg(long)]
        subject_hash: String,

        /// Subject name
        #[arg(long)]
        subject_name: String,

        /// Subject type: skill, agent, artifact, data
        #[arg(long, default_value = "skill")]
        subject_type: String,

        /// Attestation type: security_audit, code_review, functional_test, vouch
        #[arg(long, short = 't', default_value = "vouch")]
        attestation_type: String,

        /// Your agent ID
        #[arg(long)]
        agent_id: String,

        /// Your agent name
        #[arg(long)]
        agent_name: String,

        /// Platform (e.g., moltbook)
        #[arg(long)]
        platform: Option<String>,

        /// Secret key file (or - for stdin)
        #[arg(long, short = 'k')]
        key: PathBuf,

        /// Add a claim (can be repeated): --claim "no_network_exfiltration=true"
        #[arg(long, short = 'c')]
        claim: Vec<String>,

        /// Evidence method (e.g., "yara_scan", "manual_review")
        #[arg(long)]
        evidence_method: Option<String>,

        /// Evidence notes
        #[arg(long)]
        evidence_notes: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify an attestation signature
    Verify {
        /// Attestation JSON file
        file: PathBuf,

        /// Public key file (if not provided, just checks structure)
        #[arg(long, short = 'k')]
        public_key: Option<PathBuf>,
    },

    /// Check trust status for a subject
    Trust {
        /// Subject content hash
        hash: String,

        /// Attestation store file (JSON array of attestations)
        #[arg(long, short = 's')]
        store: PathBuf,

        /// Trust anchor agent IDs (can be repeated)
        #[arg(long, short = 'a')]
        anchor: Vec<String>,

        /// Minimum attestations required
        #[arg(long, default_value = "1")]
        min: usize,

        /// Require security_audit type
        #[arg(long)]
        require_audit: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen { output, public } => cmd_keygen(output, public),
        Commands::Hash { file } => cmd_hash(file),
        Commands::Attest {
            subject_hash,
            subject_name,
            subject_type,
            attestation_type,
            agent_id,
            agent_name,
            platform,
            key,
            claim,
            evidence_method,
            evidence_notes,
            output,
        } => cmd_attest(
            subject_hash,
            subject_name,
            subject_type,
            attestation_type,
            agent_id,
            agent_name,
            platform,
            key,
            claim,
            evidence_method,
            evidence_notes,
            output,
        ),
        Commands::Verify { file, public_key } => cmd_verify(file, public_key),
        Commands::Trust {
            hash,
            store,
            anchor,
            min,
            require_audit,
        } => cmd_trust(hash, store, anchor, min, require_audit),
    }
}

fn cmd_keygen(output: Option<PathBuf>, public: Option<PathBuf>) -> Result<()> {
    let keypair = KeyPair::generate();
    let secret = keypair.secret_key_base64();
    let pubkey = keypair.public_key().to_base64();

    let wrote_to_file = output.is_some();

    // Output secret key
    match output {
        Some(path) => {
            fs::write(&path, format!("{}\n", secret))
                .with_context(|| format!("Failed to write secret key to {:?}", path))?;
            eprintln!("Secret key written to {:?}", path);
        }
        None => {
            println!("{}", secret);
        }
    }

    // Output public key
    match public {
        Some(path) => {
            fs::write(&path, format!("{}\n", pubkey))
                .with_context(|| format!("Failed to write public key to {:?}", path))?;
            eprintln!("Public key written to {:?}", path);
        }
        None if wrote_to_file => {
            // If secret key went to file but no public key file specified, print it
            eprintln!("Public key: {}", pubkey);
        }
        None => {
            // Both to stdout, print on separate line
            eprintln!("Public key: {}", pubkey);
        }
    }

    eprintln!("Key ID: {}", keypair.public_key_id());
    Ok(())
}

fn cmd_hash(file: PathBuf) -> Result<()> {
    let content = if file.to_string_lossy() == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        fs::read(&file).with_context(|| format!("Failed to read {:?}", file))?
    };

    let hash = Sha256::digest(&content);
    println!("sha256:{}", hex::encode(hash));
    Ok(())
}

fn cmd_attest(
    subject_hash: String,
    subject_name: String,
    subject_type: String,
    attestation_type: String,
    agent_id: String,
    agent_name: String,
    platform: Option<String>,
    key: PathBuf,
    claims: Vec<String>,
    evidence_method: Option<String>,
    evidence_notes: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    // Load secret key
    let key_content = fs::read_to_string(&key)
        .with_context(|| format!("Failed to read key file {:?}", key))?;
    let keypair = KeyPair::from_base64(key_content.trim())
        .map_err(|e| anyhow!("Invalid key: {}", e))?;

    // Parse subject type
    let subj_type = match subject_type.to_lowercase().as_str() {
        "skill" => SubjectType::Skill,
        "agent" => SubjectType::Agent,
        "artifact" => SubjectType::Artifact,
        "data" => SubjectType::Data,
        _ => return Err(anyhow!("Unknown subject type: {}", subject_type)),
    };

    // Parse attestation type
    let att_type = match attestation_type.to_lowercase().as_str() {
        "security_audit" | "securityaudit" | "audit" => AttestationType::SecurityAudit,
        "code_review" | "codereview" | "review" => AttestationType::CodeReview,
        "functional_test" | "functionaltest" | "test" => AttestationType::FunctionalTest,
        "vouch" => AttestationType::Vouch,
        "revoke" => AttestationType::Revoke,
        _ => return Err(anyhow!("Unknown attestation type: {}", attestation_type)),
    };

    // Build subject
    let subject = Subject {
        subject_type: subj_type,
        name: subject_name,
        version: None,
        source_uri: None,
        content_hash: subject_hash,
    };

    // Build attestor
    let attestor = Attestor {
        agent_id,
        agent_name,
        platform,
    };

    // Build attestation
    let mut attestation = Attestation::new(attestor, att_type, subject);

    // Add claims
    for claim_str in claims {
        let parts: Vec<&str> = claim_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid claim format: {}. Use key=true or key=false", claim_str));
        }
        let value = match parts[1].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(anyhow!("Invalid claim value: {}. Use true or false", parts[1])),
        };
        attestation = attestation.with_claim(parts[0], value);
    }

    // Add evidence
    if let Some(method) = evidence_method {
        let mut evidence = Evidence::new(method);
        if let Some(notes) = evidence_notes {
            evidence = evidence.with_notes(notes);
        }
        attestation = attestation.with_evidence(evidence);
    }

    // Sign
    attestation.sign(&keypair).map_err(|e| anyhow!("Signing failed: {}", e))?;

    // Output
    let json = serde_json::to_string_pretty(&attestation)?;
    match output {
        Some(path) => {
            fs::write(&path, format!("{}\n", json))
                .with_context(|| format!("Failed to write attestation to {:?}", path))?;
            eprintln!("Attestation written to {:?}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

fn cmd_verify(file: PathBuf, public_key: Option<PathBuf>) -> Result<()> {
    // Load attestation
    let content = fs::read_to_string(&file)
        .with_context(|| format!("Failed to read {:?}", file))?;
    let attestation: Attestation = serde_json::from_str(&content)
        .with_context(|| "Failed to parse attestation JSON")?;

    // Check if signed
    if !attestation.is_signed() {
        eprintln!("WARNING: Attestation is not signed");
        return Ok(());
    }

    // If public key provided, verify signature
    if let Some(key_path) = public_key {
        let key_content = fs::read_to_string(&key_path)
            .with_context(|| format!("Failed to read public key {:?}", key_path))?;
        let pubkey = PublicKey::from_base64(key_content.trim())
            .map_err(|e| anyhow!("Invalid public key: {}", e))?;

        match attestation.verify(&pubkey) {
            Ok(()) => {
                eprintln!("Signature VALID");
                println!("Attestation ID: {}", attestation.attestation_id);
                println!("Type: {:?}", attestation.attestation_type);
                println!("Attestor: {} ({})", attestation.attestor.agent_name, attestation.attestor.agent_id);
                println!("Subject: {} ({})", attestation.subject.name, attestation.subject.content_hash);
                println!("Timestamp: {}", attestation.timestamp);
                if !attestation.claims.is_empty() {
                    println!("Claims:");
                    for (k, v) in &attestation.claims {
                        println!("  {}: {}", k, v);
                    }
                }
            }
            Err(e) => {
                eprintln!("Signature INVALID: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Attestation is signed (no public key provided to verify)");
        println!("Attestation ID: {}", attestation.attestation_id);
        println!("Type: {:?}", attestation.attestation_type);
        println!("Attestor: {} ({})", attestation.attestor.agent_name, attestation.attestor.agent_id);
        println!("Subject: {} ({})", attestation.subject.name, attestation.subject.content_hash);
    }

    Ok(())
}

fn cmd_trust(
    hash: String,
    store_path: PathBuf,
    anchors: Vec<String>,
    min: usize,
    require_audit: bool,
) -> Result<()> {
    // Load attestation store
    let content = fs::read_to_string(&store_path)
        .with_context(|| format!("Failed to read store {:?}", store_path))?;
    let attestations: Vec<Attestation> = serde_json::from_str(&content)
        .with_context(|| "Failed to parse attestation store JSON")?;

    let mut store = MemoryStore::new();
    for att in attestations {
        store.add(att);
    }

    // Build validator
    let mut config = ChainConfig::default().min_attestations(min);
    if require_audit {
        config = config.require_security_audit();
    }

    let mut validator = ChainValidator::new(&store).with_config(config);

    // Add trust anchors
    for anchor_id in &anchors {
        validator = validator.add_anchor(TrustAnchor::new(anchor_id, anchor_id, "CLI specified"));
    }

    // Validate
    let result = validator.validate(&hash);

    if result.trusted {
        eprintln!("TRUSTED");
        println!("Attestations: {}", result.attestation_count);
        println!("Chain depth: {}", result.chain_depth);
        for att in &result.attestations {
            println!("  - {} by {} ({:?})", att.subject.name, att.attestor.agent_name, att.attestation_type);
        }
    } else {
        eprintln!("NOT TRUSTED");
        for warning in &result.warnings {
            eprintln!("  - {}", warning);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Hex encoding utility
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
