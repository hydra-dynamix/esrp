//! # Isnad: Agent-to-Agent Attestation Protocol
//!
//! A chain-of-trust protocol for the agent ecosystem.
//! Addresses the "unsigned skill.md" supply chain attack vector.
//!
//! Named after the Islamic hadith authentication methodology where a saying
//! is only as trustworthy as its chain of transmission (isnad).
//!
//! # Core Concepts
//!
//! - **Attestation**: A signed claim by one agent about another agent/skill/artifact
//! - **Isnad Chain**: Provenance chain showing who vouched for whom
//! - **Permission Manifest**: Declaration of what a skill needs access to
//!
//! # Example
//!
//! ```
//! use isnad::{Attestation, AttestationType, Subject, Attestor, Evidence, KeyPair};
//!
//! // Generate a keypair for signing
//! let keypair = KeyPair::generate();
//!
//! let attestor = Attestor {
//!     agent_id: keypair.public_key_id(),
//!     agent_name: "Rufio".to_string(),
//!     platform: Some("moltbook".to_string()),
//! };
//!
//! let mut attestation = Attestation::new(
//!     attestor,
//!     AttestationType::SecurityAudit,
//!     Subject::skill("weather-skill", "sha256:abc123..."),
//! )
//! .with_claim("no_network_exfiltration", true)
//! .with_evidence(Evidence::new("yara_scan").with_notes("No malicious patterns"));
//!
//! // Sign the attestation
//! attestation.sign(&keypair).expect("signing failed");
//!
//! // Verify later
//! assert!(attestation.verify(&keypair.public_key()).is_ok());
//! ```

mod captcha;
mod chain;
mod error;
mod reputation;
mod signing;
mod types;

pub use captcha::*;
pub use chain::*;
pub use error::*;
pub use reputation::*;
pub use signing::*;
pub use types::*;
