//! Isnad CLI - Agent trust attestation tool.
//!
//! Run `isnad` for command list, `isnad <cmd> --help` for details.

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use isnad::{
    Attestation, AttestationType, Attestor, CaptchaChallenge, CaptchaConfig, CaptchaResponse,
    CaptchaVerifier, ChainConfig, ChainValidator, Evidence, KeyPair, MemoryStore, PublicKey,
    ReputationCalculator, Subject, SubjectType, TaskAnswer, TrustAnchor,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "isnad")]
#[command(about = "Agent trust attestations. Use --help on subcommands for details.")]
#[command(version, propagate_version = true)]
struct Cli {
    /// Output JSON (machine-readable)
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Ed25519 keypair
    #[command(visible_alias = "kg")]
    Keygen {
        /// Secret key output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Public key output file
        #[arg(short, long)]
        public: Option<PathBuf>,
    },

    /// SHA256 hash a file
    Hash {
        /// File (- for stdin)
        file: PathBuf,
    },

    /// Create signed attestation
    #[command(visible_alias = "att")]
    Attest {
        /// Subject content hash
        #[arg(long)]
        subject_hash: String,
        /// Subject name
        #[arg(long)]
        subject_name: String,
        /// Type: skill|agent|artifact|data
        #[arg(long, default_value = "skill")]
        subject_type: String,
        /// Type: audit|review|test|vouch|revoke
        #[arg(long, short = 't', default_value = "vouch")]
        attestation_type: String,
        /// Your agent ID
        #[arg(long)]
        agent_id: String,
        /// Your agent name
        #[arg(long)]
        agent_name: String,
        /// Platform (e.g. moltbook)
        #[arg(long)]
        platform: Option<String>,
        /// Secret key file
        #[arg(long, short = 'k')]
        key: PathBuf,
        /// Claim: key=true|false (repeatable)
        #[arg(long, short = 'c')]
        claim: Vec<String>,
        /// Evidence method
        #[arg(long)]
        evidence_method: Option<String>,
        /// Evidence notes
        #[arg(long)]
        evidence_notes: Option<String>,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify attestation signature
    #[command(visible_alias = "v")]
    Verify {
        /// Attestation JSON file
        file: PathBuf,
        /// Public key file
        #[arg(long, short = 'k')]
        public_key: Option<PathBuf>,
    },

    /// Check trust for subject hash
    #[command(visible_alias = "t")]
    Trust {
        /// Subject content hash
        hash: String,
        /// Attestation store JSON
        #[arg(long, short = 's')]
        store: PathBuf,
        /// Trust anchor agent ID (repeatable)
        #[arg(long, short = 'a')]
        anchor: Vec<String>,
        /// Minimum attestations
        #[arg(long, default_value = "1")]
        min: usize,
        /// Require security_audit
        #[arg(long)]
        require_audit: bool,
    },

    /// Compute reputation score
    #[command(visible_alias = "rep")]
    Reputation {
        /// Subject content hash
        hash: String,
        /// Attestation store JSON
        #[arg(long, short = 's')]
        store: PathBuf,
        /// Trust anchor agent ID (repeatable)
        #[arg(long, short = 'a')]
        anchor: Vec<String>,
    },

    /// AI CAPTCHA - prove you're an agent
    #[command(visible_alias = "cap")]
    Captcha {
        #[command(subcommand)]
        action: CaptchaAction,
    },
}

#[derive(Subcommand)]
enum CaptchaAction {
    /// Generate a challenge (save expected answers separately)
    Generate {
        /// Output challenge file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output expected answers file (keep secret!)
        #[arg(long)]
        answers: Option<PathBuf>,
        /// Time limit in milliseconds
        #[arg(long, default_value = "5000")]
        time_limit: u64,
    },

    /// Verify a response against challenge + answers
    Verify {
        /// Challenge JSON file
        #[arg(long, short = 'c')]
        challenge: PathBuf,
        /// Response JSON file
        #[arg(long, short = 'r')]
        response: PathBuf,
        /// Expected answers JSON file
        #[arg(long, short = 'a')]
        answers: PathBuf,
    },
}

// JSON output types
#[derive(Serialize)]
struct KeygenOutput {
    secret_key: String,
    public_key: String,
    key_id: String,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
}

#[derive(Serialize)]
struct VerifyOutput {
    valid: bool,
    attestation_id: String,
    attestation_type: String,
    attestor: String,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct TrustOutput {
    trusted: bool,
    attestation_count: usize,
    chain_depth: usize,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ReputationOutput {
    score: f64,
    attestation_count: usize,
    avg_age_days: f64,
    warnings: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen { output, public } => cmd_keygen(cli.json, output, public),
        Commands::Hash { file } => cmd_hash(cli.json, file),
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
            cli.json,
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
        Commands::Verify { file, public_key } => cmd_verify(cli.json, file, public_key),
        Commands::Trust {
            hash,
            store,
            anchor,
            min,
            require_audit,
        } => cmd_trust(cli.json, hash, store, anchor, min, require_audit),
        Commands::Reputation {
            hash,
            store,
            anchor,
        } => cmd_reputation(cli.json, hash, store, anchor),
        Commands::Captcha { action } => match action {
            CaptchaAction::Generate {
                output,
                answers,
                time_limit,
            } => cmd_captcha_generate(cli.json, output, answers, time_limit),
            CaptchaAction::Verify {
                challenge,
                response,
                answers,
            } => cmd_captcha_verify(cli.json, challenge, response, answers),
        },
    }
}

fn cmd_keygen(json: bool, output: Option<PathBuf>, public: Option<PathBuf>) -> Result<()> {
    let keypair = KeyPair::generate();
    let secret = keypair.secret_key_base64();
    let pubkey = keypair.public_key().to_base64();
    let key_id = keypair.public_key_id();

    if json {
        let out = KeygenOutput {
            secret_key: secret.clone(),
            public_key: pubkey.clone(),
            key_id: key_id.clone(),
        };
        println!("{}", serde_json::to_string(&out)?);
    }

    let wrote_to_file = output.is_some();

    match output {
        Some(path) => {
            fs::write(&path, format!("{}\n", secret))
                .with_context(|| format!("Failed to write secret key to {:?}", path))?;
            if !json {
                eprintln!("secret_key: {:?}", path);
            }
        }
        None if !json => {
            println!("{}", secret);
        }
        None => {}
    }

    match public {
        Some(path) => {
            fs::write(&path, format!("{}\n", pubkey))
                .with_context(|| format!("Failed to write public key to {:?}", path))?;
            if !json {
                eprintln!("public_key: {:?}", path);
            }
        }
        None if wrote_to_file && !json => {
            eprintln!("public_key: {}", pubkey);
        }
        None if !json => {
            eprintln!("public_key: {}", pubkey);
        }
        None => {}
    }

    if !json {
        eprintln!("key_id: {}", key_id);
    }
    Ok(())
}

fn cmd_hash(json: bool, file: PathBuf) -> Result<()> {
    let content = if file.to_string_lossy() == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        fs::read(&file).with_context(|| format!("Failed to read {:?}", file))?
    };

    let hash = Sha256::digest(&content);
    let hash_str = format!("sha256:{}", hex::encode(hash));

    if json {
        println!("{}", serde_json::to_string(&HashOutput { hash: hash_str })?);
    } else {
        println!("{}", hash_str);
    }
    Ok(())
}

fn cmd_attest(
    _json: bool, // Attestation output is always JSON
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
    let key_content =
        fs::read_to_string(&key).with_context(|| format!("Failed to read key file {:?}", key))?;
    let keypair =
        KeyPair::from_base64(key_content.trim()).map_err(|e| anyhow!("Invalid key: {}", e))?;

    let subj_type = match subject_type.to_lowercase().as_str() {
        "skill" => SubjectType::Skill,
        "agent" => SubjectType::Agent,
        "artifact" => SubjectType::Artifact,
        "data" => SubjectType::Data,
        _ => return Err(anyhow!("subject_type: skill|agent|artifact|data")),
    };

    let att_type = match attestation_type.to_lowercase().as_str() {
        "security_audit" | "securityaudit" | "audit" => AttestationType::SecurityAudit,
        "code_review" | "codereview" | "review" => AttestationType::CodeReview,
        "functional_test" | "functionaltest" | "test" => AttestationType::FunctionalTest,
        "vouch" => AttestationType::Vouch,
        "revoke" => AttestationType::Revoke,
        _ => return Err(anyhow!("attestation_type: audit|review|test|vouch|revoke")),
    };

    let subject = Subject {
        subject_type: subj_type,
        name: subject_name,
        version: None,
        source_uri: None,
        content_hash: subject_hash,
    };

    let attestor = Attestor {
        agent_id,
        agent_name,
        platform,
    };

    let mut attestation = Attestation::new(attestor, att_type, subject);

    for claim_str in claims {
        let parts: Vec<&str> = claim_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("claim format: key=true|false"));
        }
        let value = matches!(parts[1].to_lowercase().as_str(), "true" | "1" | "yes");
        attestation = attestation.with_claim(parts[0], value);
    }

    if let Some(method) = evidence_method {
        let mut evidence = Evidence::new(method);
        if let Some(notes) = evidence_notes {
            evidence = evidence.with_notes(notes);
        }
        attestation = attestation.with_evidence(evidence);
    }

    attestation
        .sign(&keypair)
        .map_err(|e| anyhow!("Signing failed: {}", e))?;

    let out = serde_json::to_string_pretty(&attestation)?;
    match output {
        Some(path) => {
            fs::write(&path, format!("{}\n", out))
                .with_context(|| format!("Failed to write attestation to {:?}", path))?;
            eprintln!("wrote: {:?}", path);
        }
        None => {
            println!("{}", out);
        }
    }

    Ok(())
}

fn cmd_verify(json: bool, file: PathBuf, public_key: Option<PathBuf>) -> Result<()> {
    let content =
        fs::read_to_string(&file).with_context(|| format!("Failed to read {:?}", file))?;
    let attestation: Attestation =
        serde_json::from_str(&content).with_context(|| "Failed to parse attestation JSON")?;

    if !attestation.is_signed() {
        if json {
            println!(
                "{}",
                serde_json::to_string(&VerifyOutput {
                    valid: false,
                    attestation_id: attestation.attestation_id.to_string(),
                    attestation_type: format!("{:?}", attestation.attestation_type),
                    attestor: attestation.attestor.agent_name.clone(),
                    subject: attestation.subject.name.clone(),
                    error: Some("not signed".to_string()),
                })?
            );
        } else {
            eprintln!("WARN: not signed");
        }
        return Ok(());
    }

    if let Some(key_path) = public_key {
        let key_content = fs::read_to_string(&key_path)
            .with_context(|| format!("Failed to read public key {:?}", key_path))?;
        let pubkey = PublicKey::from_base64(key_content.trim())
            .map_err(|e| anyhow!("Invalid public key: {}", e))?;

        match attestation.verify(&pubkey) {
            Ok(()) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string(&VerifyOutput {
                            valid: true,
                            attestation_id: attestation.attestation_id.to_string(),
                            attestation_type: format!("{:?}", attestation.attestation_type),
                            attestor: attestation.attestor.agent_name.clone(),
                            subject: attestation.subject.name.clone(),
                            error: None,
                        })?
                    );
                } else {
                    println!("VALID");
                    println!("id: {}", attestation.attestation_id);
                    println!("type: {:?}", attestation.attestation_type);
                    println!("attestor: {}", attestation.attestor.agent_name);
                    println!("subject: {}", attestation.subject.name);
                }
            }
            Err(e) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string(&VerifyOutput {
                            valid: false,
                            attestation_id: attestation.attestation_id.to_string(),
                            attestation_type: format!("{:?}", attestation.attestation_type),
                            attestor: attestation.attestor.agent_name.clone(),
                            subject: attestation.subject.name.clone(),
                            error: Some(e.to_string()),
                        })?
                    );
                } else {
                    eprintln!("INVALID: {}", e);
                }
                std::process::exit(1);
            }
        }
    } else if json {
        println!(
            "{}",
            serde_json::to_string(&VerifyOutput {
                valid: true, // signed but unverified
                attestation_id: attestation.attestation_id.to_string(),
                attestation_type: format!("{:?}", attestation.attestation_type),
                attestor: attestation.attestor.agent_name.clone(),
                subject: attestation.subject.name.clone(),
                error: None,
            })?
        );
    } else {
        println!("signed (no key to verify)");
        println!("id: {}", attestation.attestation_id);
    }

    Ok(())
}

fn cmd_trust(
    json: bool,
    hash: String,
    store_path: PathBuf,
    anchors: Vec<String>,
    min: usize,
    require_audit: bool,
) -> Result<()> {
    let content = fs::read_to_string(&store_path)
        .with_context(|| format!("Failed to read store {:?}", store_path))?;
    let attestations: Vec<Attestation> =
        serde_json::from_str(&content).with_context(|| "Failed to parse attestation store")?;

    let mut store = MemoryStore::new();
    for att in attestations {
        store.add(att);
    }

    let mut config = ChainConfig::default().min_attestations(min);
    if require_audit {
        config = config.require_security_audit();
    }

    let mut validator = ChainValidator::new(&store).with_config(config);
    for anchor_id in &anchors {
        validator = validator.add_anchor(TrustAnchor::new(anchor_id, anchor_id, "cli"));
    }

    let result = validator.validate(&hash);

    if json {
        println!(
            "{}",
            serde_json::to_string(&TrustOutput {
                trusted: result.trusted,
                attestation_count: result.attestation_count,
                chain_depth: result.chain_depth,
                warnings: result.warnings.clone(),
            })?
        );
    } else if result.trusted {
        println!("TRUSTED");
        println!("attestations: {}", result.attestation_count);
        println!("chain_depth: {}", result.chain_depth);
    } else {
        eprintln!("NOT_TRUSTED");
        for w in &result.warnings {
            eprintln!("  {}", w);
        }
    }

    if !result.trusted {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_reputation(
    json: bool,
    hash: String,
    store_path: PathBuf,
    anchors: Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string(&store_path)
        .with_context(|| format!("Failed to read store {:?}", store_path))?;
    let attestations: Vec<Attestation> =
        serde_json::from_str(&content).with_context(|| "Failed to parse attestation store")?;

    let mut store = MemoryStore::new();
    for att in attestations {
        store.add(att);
    }

    let mut calc = ReputationCalculator::new(&store);
    for anchor_id in &anchors {
        calc = calc.add_anchor(TrustAnchor::new(anchor_id, anchor_id, "cli"));
    }

    let result = calc.compute(&hash);

    if json {
        println!(
            "{}",
            serde_json::to_string(&ReputationOutput {
                score: result.score,
                attestation_count: result.attestation_count,
                avg_age_days: result.avg_age_days,
                warnings: result.warnings.clone(),
            })?
        );
    } else {
        println!("score: {:.2}", result.score);
        println!("attestations: {}", result.attestation_count);
        println!("avg_age_days: {:.1}", result.avg_age_days);
        if !result.warnings.is_empty() {
            for w in &result.warnings {
                eprintln!("WARN: {}", w);
            }
        }
    }

    Ok(())
}

fn cmd_captcha_generate(
    _json: bool, // Always JSON output for captcha
    output: Option<PathBuf>,
    answers_path: Option<PathBuf>,
    time_limit: u64,
) -> Result<()> {
    let config = CaptchaConfig {
        time_limit_ms: time_limit,
        ..Default::default()
    };
    let verifier = CaptchaVerifier::with_config(config);
    let (challenge, expected_answers) = verifier.generate_challenge();

    // Output challenge
    let challenge_json = serde_json::to_string_pretty(&challenge)?;
    match output {
        Some(path) => {
            fs::write(&path, format!("{}\n", challenge_json))
                .with_context(|| format!("Failed to write challenge to {:?}", path))?;
            eprintln!("challenge: {:?}", path);
        }
        None => {
            println!("{}", challenge_json);
        }
    }

    // Output expected answers (keep secret!)
    match answers_path {
        Some(path) => {
            let answers_json = serde_json::to_string_pretty(&expected_answers)?;
            fs::write(&path, format!("{}\n", answers_json))
                .with_context(|| format!("Failed to write answers to {:?}", path))?;
            eprintln!("answers: {:?} (keep secret!)", path);
        }
        None => {
            eprintln!("WARN: answers not saved. Use --answers to save for verification.");
        }
    }

    eprintln!("time_limit_ms: {}", time_limit);
    eprintln!("challenge_id: {}", challenge.challenge_id);

    Ok(())
}

fn cmd_captcha_verify(
    json: bool,
    challenge_path: PathBuf,
    response_path: PathBuf,
    answers_path: PathBuf,
) -> Result<()> {
    // Load challenge
    let challenge_content = fs::read_to_string(&challenge_path)
        .with_context(|| format!("Failed to read challenge {:?}", challenge_path))?;
    let challenge: CaptchaChallenge = serde_json::from_str(&challenge_content)
        .with_context(|| "Failed to parse challenge JSON")?;

    // Load response
    let response_content = fs::read_to_string(&response_path)
        .with_context(|| format!("Failed to read response {:?}", response_path))?;
    let response: CaptchaResponse = serde_json::from_str(&response_content)
        .with_context(|| "Failed to parse response JSON")?;

    // Load expected answers
    let answers_content = fs::read_to_string(&answers_path)
        .with_context(|| format!("Failed to read answers {:?}", answers_path))?;
    let expected: Vec<TaskAnswer> = serde_json::from_str(&answers_content)
        .with_context(|| "Failed to parse answers JSON")?;

    // Verify
    let verifier = CaptchaVerifier::new();
    match verifier.verify(&challenge, &response, &expected) {
        Ok(verification) => {
            if json {
                println!("{}", serde_json::to_string(&verification)?);
            } else {
                println!("VERIFIED");
                println!("elapsed_ms: {}", verification.elapsed_ms);
                println!(
                    "tasks: {}/{}",
                    verification.tasks_correct, verification.tasks_total
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::json!({
                        "verified": false,
                        "error": e.to_string()
                    }))?
                );
            } else {
                eprintln!("FAILED: {}", e);
            }
            std::process::exit(1);
        }
    }
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
