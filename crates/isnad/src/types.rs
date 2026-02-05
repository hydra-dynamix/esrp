//! Core types for ESRP-Trust attestations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of attestation being made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationType {
    /// Code reviewed for malicious behavior (highest trust signal)
    SecurityAudit,
    /// Code quality/correctness review
    CodeReview,
    /// Tested and works as described
    FunctionalTest,
    /// General endorsement without specific claims
    Vouch,
    /// Withdraw a previous attestation
    Revoke,
}

/// What kind of thing is being attested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectType {
    /// A skill.md or similar executable content
    Skill,
    /// An agent's identity
    Agent,
    /// A code artifact (library, binary)
    Artifact,
    /// A data artifact
    Data,
}

/// The subject of an attestation - what's being vouched for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Type of subject
    #[serde(rename = "type")]
    pub subject_type: SubjectType,

    /// Human-readable name
    pub name: String,

    /// Version if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Source URI where this can be retrieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,

    /// SHA256 hash of content (required for verification)
    pub content_hash: String,
}

impl Subject {
    /// Create a subject for a skill
    pub fn skill(name: impl Into<String>, content_hash: impl Into<String>) -> Self {
        Self {
            subject_type: SubjectType::Skill,
            name: name.into(),
            version: None,
            source_uri: None,
            content_hash: content_hash.into(),
        }
    }

    /// Create a subject for an agent
    pub fn agent(name: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            subject_type: SubjectType::Agent,
            name: name.into(),
            version: None,
            source_uri: None,
            content_hash: agent_id.into(), // For agents, this is their ID
        }
    }

    /// Add version information
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add source URI
    pub fn with_source(mut self, uri: impl Into<String>) -> Self {
        self.source_uri = Some(uri.into());
        self
    }
}

/// Identity of the agent making the attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestor {
    /// Unique agent ID (typically UUID)
    pub agent_id: String,

    /// Human-readable agent name
    pub agent_name: String,

    /// Platform where agent is registered (e.g., "moltbook")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

/// Evidence supporting the attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Method used (e.g., "yara_scan", "manual_review", "automated_test")
    pub method: String,

    /// Tool/version used if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,

    /// Human-readable notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// URI to detailed report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_uri: Option<String>,

    /// Additional structured data
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Evidence {
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            tool_version: None,
            notes: None,
            report_uri: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool_version = Some(tool.into());
        self
    }

    pub fn with_report(mut self, uri: impl Into<String>) -> Self {
        self.report_uri = Some(uri.into());
        self
    }
}

/// A signed attestation from one agent about a subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Unique attestation ID
    pub attestation_id: Uuid,

    /// Type of attestation
    pub attestation_type: AttestationType,

    /// When the attestation was made
    pub timestamp: DateTime<Utc>,

    /// Who is making the attestation
    pub attestor: Attestor,

    /// What is being attested
    pub subject: Subject,

    /// Specific claims being made
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub claims: HashMap<String, bool>,

    /// Evidence supporting the claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Evidence>,

    /// Link to previous attestation in the chain (isnad)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<Uuid>,

    /// Cryptographic signature (Ed25519 recommended)
    /// Format: "ed25519:<base64-signature>"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Attestation {
    /// Create a new unsigned attestation
    pub fn new(attestor: Attestor, attestation_type: AttestationType, subject: Subject) -> Self {
        Self {
            attestation_id: Uuid::new_v4(),
            attestation_type,
            timestamp: Utc::now(),
            attestor,
            subject,
            claims: HashMap::new(),
            evidence: None,
            causation_id: None,
            signature: None,
        }
    }

    /// Add a boolean claim
    pub fn with_claim(mut self, claim: impl Into<String>, value: bool) -> Self {
        self.claims.insert(claim.into(), value);
        self
    }

    /// Add evidence
    pub fn with_evidence(mut self, evidence: Evidence) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// Link to a previous attestation (isnad chain)
    pub fn with_causation(mut self, previous: Uuid) -> Self {
        self.causation_id = Some(previous);
        self
    }

    /// Check if attestation is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Standard claims for security audits
    pub fn security_audit_claims() -> Vec<&'static str> {
        vec![
            "no_network_exfiltration",
            "no_credential_access",
            "no_filesystem_outside_workspace",
            "no_subprocess_spawn",
            "permissions_declared_complete",
        ]
    }
}

/// Permission manifest declaring what a skill needs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionManifest {
    /// Skill identifier
    pub skill: String,

    /// Skill version
    pub version: String,

    /// Network permissions
    #[serde(default)]
    pub network: NetworkPermissions,

    /// Filesystem permissions
    #[serde(default)]
    pub filesystem: FilesystemPermissions,

    /// Environment variable access
    #[serde(default)]
    pub environment: EnvironmentPermissions,

    /// Can spawn subprocesses
    #[serde(default)]
    pub subprocess: bool,

    /// Contains native code
    #[serde(default)]
    pub native_code: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Specific hosts allowed
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// Allow connections to any host
    #[serde(default)]
    pub allow_arbitrary: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    /// Paths that can be read
    #[serde(default)]
    pub read: Vec<String>,

    /// Paths that can be written
    #[serde(default)]
    pub write: Vec<String>,

    /// Allow access outside declared paths
    #[serde(default)]
    pub allow_arbitrary: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentPermissions {
    /// Environment variables that can be read
    #[serde(default)]
    pub read: Vec<String>,

    /// Allow reading any environment variable
    #[serde(default)]
    pub allow_arbitrary: bool,
}

/// Result of a trust query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustResult {
    /// Whether the subject meets trust requirements
    pub trusted: bool,

    /// Number of attestations found
    pub attestation_count: usize,

    /// The attestations themselves
    pub attestations: Vec<Attestation>,

    /// How deep the isnad chain goes
    pub chain_depth: usize,

    /// Any warnings (e.g., old attestations, revocations)
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl TrustResult {
    pub fn untrusted(reason: impl Into<String>) -> Self {
        Self {
            trusted: false,
            attestation_count: 0,
            attestations: vec![],
            chain_depth: 0,
            warnings: vec![reason.into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_attestation() {
        let attestor = Attestor {
            agent_id: "123".to_string(),
            agent_name: "Rufio".to_string(),
            platform: Some("moltbook".to_string()),
        };

        let subject = Subject::skill("weather-skill", "sha256:abc123")
            .with_version("1.0.0");

        let attestation = Attestation::new(
            attestor,
            AttestationType::SecurityAudit,
            subject,
        )
        .with_claim("no_network_exfiltration", true)
        .with_claim("permissions_declared_complete", true)
        .with_evidence(
            Evidence::new("yara_scan")
                .with_tool("yara 4.3.0")
                .with_notes("No malicious patterns found")
        );

        assert_eq!(attestation.attestation_type, AttestationType::SecurityAudit);
        assert_eq!(attestation.claims.len(), 2);
        assert!(attestation.evidence.is_some());
        assert!(!attestation.is_signed());
    }

    #[test]
    fn test_serialize_attestation() {
        let attestor = Attestor {
            agent_id: "123".to_string(),
            agent_name: "Rufio".to_string(),
            platform: None,
        };

        let attestation = Attestation::new(
            attestor,
            AttestationType::Vouch,
            Subject::agent("eudaemon_0", "456"),
        );

        let json = serde_json::to_string_pretty(&attestation).unwrap();
        assert!(json.contains("\"attestation_type\": \"vouch\""));
        assert!(json.contains("\"agent_name\": \"Rufio\""));
    }

    #[test]
    fn test_permission_manifest() {
        let manifest = PermissionManifest {
            skill: "weather-skill".to_string(),
            version: "1.0.0".to_string(),
            network: NetworkPermissions {
                allowed_hosts: vec!["api.openweathermap.org".to_string()],
                allow_arbitrary: false,
            },
            filesystem: FilesystemPermissions {
                read: vec!["workspace://config/".to_string()],
                write: vec!["workspace://cache/".to_string()],
                allow_arbitrary: false,
            },
            environment: EnvironmentPermissions {
                read: vec!["WEATHER_API_KEY".to_string()],
                allow_arbitrary: false,
            },
            subprocess: false,
            native_code: false,
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("api.openweathermap.org"));
        assert!(json.contains("WEATHER_API_KEY"));
    }
}
