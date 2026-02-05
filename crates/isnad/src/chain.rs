//! Isnad chain validation - traverse attestation chains to trust anchors.
//!
//! # Concept
//!
//! An isnad chain traces who vouched for whom:
//!
//! ```text
//! Skill "weather-skill"
//!   └─ Attested by: Rufio (security_audit)
//!        └─ Rufio vouched by: eudaemon_0 (vouch)
//!             └─ eudaemon_0 is a trust anchor
//! ```
//!
//! Chain validation answers: "Does this attestation trace back to someone I trust?"

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::IsnadError;
use crate::signing::PublicKey;
use crate::types::{Attestation, AttestationType, TrustResult};

/// Configuration for chain validation.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// Maximum depth to traverse (default: 5)
    pub max_depth: usize,

    /// Minimum attestations required for trust (default: 1)
    pub min_attestations: usize,

    /// Required attestation types (empty = any type accepted)
    pub required_types: Vec<AttestationType>,

    /// How old an attestation can be before warning (in days, 0 = no limit)
    pub max_age_days: u64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            min_attestations: 1,
            required_types: vec![],
            max_age_days: 0,
        }
    }
}

impl ChainConfig {
    /// Require security audits specifically.
    pub fn require_security_audit(mut self) -> Self {
        self.required_types = vec![AttestationType::SecurityAudit];
        self
    }

    /// Set minimum attestation count.
    pub fn min_attestations(mut self, n: usize) -> Self {
        self.min_attestations = n;
        self
    }

    /// Set maximum chain depth.
    pub fn max_depth(mut self, n: usize) -> Self {
        self.max_depth = n;
        self
    }
}

/// A trust anchor - an agent trusted without needing further vouching.
#[derive(Debug, Clone)]
pub struct TrustAnchor {
    /// Agent ID (matches attestor.agent_id)
    pub agent_id: String,

    /// Human-readable name
    pub name: String,

    /// Public key for signature verification (optional)
    pub public_key: Option<PublicKey>,

    /// Why this agent is trusted
    pub reason: String,
}

impl TrustAnchor {
    pub fn new(agent_id: impl Into<String>, name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            public_key: None,
            reason: reason.into(),
        }
    }

    pub fn with_public_key(mut self, key: PublicKey) -> Self {
        self.public_key = Some(key);
        self
    }
}

/// Store for attestations - trait to allow different backends.
pub trait AttestationStore {
    /// Get attestation by ID.
    fn get(&self, id: &Uuid) -> Option<&Attestation>;

    /// Find attestations for a subject (by content hash).
    fn find_by_subject(&self, content_hash: &str) -> Vec<&Attestation>;

    /// Find attestations made by an agent.
    fn find_by_attestor(&self, agent_id: &str) -> Vec<&Attestation>;

    /// Find vouches for an agent (attestations where subject is this agent).
    fn find_vouches_for(&self, agent_id: &str) -> Vec<&Attestation>;

    /// Check if an attestation has been revoked.
    fn is_revoked(&self, id: &Uuid) -> bool;
}

/// In-memory attestation store for testing and simple use cases.
#[derive(Debug, Default)]
pub struct MemoryStore {
    attestations: HashMap<Uuid, Attestation>,
    revoked: HashSet<Uuid>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attestation to the store.
    pub fn add(&mut self, attestation: Attestation) {
        // If this is a revocation, mark the causation as revoked
        if attestation.attestation_type == AttestationType::Revoke {
            if let Some(target) = attestation.causation_id {
                self.revoked.insert(target);
            }
        }
        self.attestations.insert(attestation.attestation_id, attestation);
    }

    /// Get all attestations.
    pub fn all(&self) -> impl Iterator<Item = &Attestation> {
        self.attestations.values()
    }
}

impl AttestationStore for MemoryStore {
    fn get(&self, id: &Uuid) -> Option<&Attestation> {
        self.attestations.get(id)
    }

    fn find_by_subject(&self, content_hash: &str) -> Vec<&Attestation> {
        self.attestations
            .values()
            .filter(|a| a.subject.content_hash == content_hash)
            .filter(|a| a.attestation_type != AttestationType::Revoke) // Revocations aren't positive attestations
            .filter(|a| !self.revoked.contains(&a.attestation_id))
            .collect()
    }

    fn find_by_attestor(&self, agent_id: &str) -> Vec<&Attestation> {
        self.attestations
            .values()
            .filter(|a| a.attestor.agent_id == agent_id)
            .filter(|a| !self.revoked.contains(&a.attestation_id))
            .collect()
    }

    fn find_vouches_for(&self, agent_id: &str) -> Vec<&Attestation> {
        self.attestations
            .values()
            .filter(|a| {
                a.subject.content_hash == agent_id
                    && (a.attestation_type == AttestationType::Vouch
                        || a.attestation_type == AttestationType::SecurityAudit
                        || a.attestation_type == AttestationType::CodeReview)
            })
            .filter(|a| !self.revoked.contains(&a.attestation_id))
            .collect()
    }

    fn is_revoked(&self, id: &Uuid) -> bool {
        self.revoked.contains(id)
    }
}

/// Chain validator - traverses isnad chains to trust anchors.
pub struct ChainValidator<'a, S: AttestationStore> {
    store: &'a S,
    anchors: Vec<TrustAnchor>,
    config: ChainConfig,
}

impl<'a, S: AttestationStore> ChainValidator<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self {
            store,
            anchors: vec![],
            config: ChainConfig::default(),
        }
    }

    /// Add a trust anchor.
    pub fn add_anchor(mut self, anchor: TrustAnchor) -> Self {
        self.anchors.push(anchor);
        self
    }

    /// Set validation config.
    pub fn with_config(mut self, config: ChainConfig) -> Self {
        self.config = config;
        self
    }

    /// Check if an agent is a trust anchor.
    fn is_anchor(&self, agent_id: &str) -> bool {
        self.anchors.iter().any(|a| a.agent_id == agent_id)
    }

    /// Validate trust for a subject (by content hash).
    pub fn validate(&self, content_hash: &str) -> TrustResult {
        let attestations = self.store.find_by_subject(content_hash);

        if attestations.is_empty() {
            return TrustResult::untrusted("no attestations found");
        }

        // Filter by required types if specified
        let filtered: Vec<_> = if self.config.required_types.is_empty() {
            attestations
        } else {
            attestations
                .into_iter()
                .filter(|a| self.config.required_types.contains(&a.attestation_type))
                .collect()
        };

        let filtered_count = filtered.len();
        if filtered_count < self.config.min_attestations {
            return TrustResult {
                trusted: false,
                attestation_count: filtered_count,
                attestations: filtered.into_iter().cloned().collect(),
                chain_depth: 0,
                warnings: vec![format!(
                    "need {} attestations, found {}",
                    self.config.min_attestations,
                    filtered_count
                )],
            };
        }

        // Check each attestation's chain
        let mut valid_attestations = vec![];
        let mut max_depth = 0;
        let mut warnings = vec![];

        for attestation in filtered {
            match self.trace_chain(&attestation.attestor.agent_id, 0, &mut HashSet::new()) {
                Ok(depth) => {
                    valid_attestations.push(attestation.clone());
                    max_depth = max_depth.max(depth);
                }
                Err(e) => {
                    warnings.push(format!(
                        "chain for {} failed: {}",
                        attestation.attestor.agent_name, e
                    ));
                }
            }
        }

        // Check age warnings
        let now = chrono::Utc::now();
        if self.config.max_age_days > 0 {
            for att in &valid_attestations {
                let age = now.signed_duration_since(att.timestamp);
                if age.num_days() > self.config.max_age_days as i64 {
                    warnings.push(format!(
                        "attestation {} is {} days old",
                        att.attestation_id,
                        age.num_days()
                    ));
                }
            }
        }

        if valid_attestations.len() < self.config.min_attestations {
            TrustResult {
                trusted: false,
                attestation_count: valid_attestations.len(),
                attestations: valid_attestations,
                chain_depth: max_depth,
                warnings,
            }
        } else {
            TrustResult {
                trusted: true,
                attestation_count: valid_attestations.len(),
                attestations: valid_attestations,
                chain_depth: max_depth,
                warnings,
            }
        }
    }

    /// Trace an attestor's chain back to a trust anchor.
    /// Returns the depth if successful.
    fn trace_chain(
        &self,
        agent_id: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Result<usize, IsnadError> {
        // Check depth limit
        if depth > self.config.max_depth {
            return Err(IsnadError::ChainValidationFailed(format!(
                "max depth {} exceeded",
                self.config.max_depth
            )));
        }

        // Cycle detection
        if visited.contains(agent_id) {
            return Err(IsnadError::ChainValidationFailed(
                "cycle detected in chain".to_string(),
            ));
        }
        visited.insert(agent_id.to_string());

        // Is this agent a trust anchor?
        if self.is_anchor(agent_id) {
            return Ok(depth);
        }

        // Find vouches for this agent
        let vouches = self.store.find_vouches_for(agent_id);

        if vouches.is_empty() {
            return Err(IsnadError::ChainValidationFailed(format!(
                "agent {} has no vouches and is not a trust anchor",
                agent_id
            )));
        }

        // Try to trace any of the vouchers
        for vouch in vouches {
            if let Ok(d) = self.trace_chain(&vouch.attestor.agent_id, depth + 1, visited) {
                return Ok(d);
            }
        }

        Err(IsnadError::ChainValidationFailed(format!(
            "no valid chain found for agent {}",
            agent_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Attestor, Subject};

    fn make_attestor(id: &str, name: &str) -> Attestor {
        Attestor {
            agent_id: id.to_string(),
            agent_name: name.to_string(),
            platform: Some("test".to_string()),
        }
    }

    #[test]
    fn test_simple_trust_anchor() {
        let mut store = MemoryStore::new();

        // Rufio attests weather-skill
        let attestation = Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        );
        store.add(attestation);

        // Rufio is a trust anchor
        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("rufio-id", "Rufio", "platform verified"));

        let result = validator.validate("sha256:abc123");
        assert!(result.trusted);
        assert_eq!(result.attestation_count, 1);
        assert_eq!(result.chain_depth, 0); // Direct anchor
    }

    #[test]
    fn test_chain_traversal() {
        let mut store = MemoryStore::new();

        // Rufio attests weather-skill
        let skill_attestation = Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        );
        store.add(skill_attestation);

        // eudaemon_0 vouches for Rufio
        let vouch = Attestation::new(
            make_attestor("eudaemon-id", "eudaemon_0"),
            AttestationType::Vouch,
            Subject::agent("Rufio", "rufio-id"),
        );
        store.add(vouch);

        // eudaemon_0 is the trust anchor
        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("eudaemon-id", "eudaemon_0", "OG security researcher"));

        let result = validator.validate("sha256:abc123");
        assert!(result.trusted);
        assert_eq!(result.chain_depth, 1); // One hop to anchor
    }

    #[test]
    fn test_deeper_chain() {
        let mut store = MemoryStore::new();

        // NewAgent attests weather-skill
        store.add(Attestation::new(
            make_attestor("new-agent", "NewAgent"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        ));

        // Rufio vouches for NewAgent
        store.add(Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::Vouch,
            Subject::agent("NewAgent", "new-agent"),
        ));

        // eudaemon_0 vouches for Rufio
        store.add(Attestation::new(
            make_attestor("eudaemon-id", "eudaemon_0"),
            AttestationType::Vouch,
            Subject::agent("Rufio", "rufio-id"),
        ));

        // eudaemon_0 is anchor
        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("eudaemon-id", "eudaemon_0", "trusted"));

        let result = validator.validate("sha256:abc123");
        assert!(result.trusted);
        assert_eq!(result.chain_depth, 2);
    }

    #[test]
    fn test_no_attestations() {
        let store = MemoryStore::new();
        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("anyone", "Anyone", "doesn't matter"));

        let result = validator.validate("sha256:unknown");
        assert!(!result.trusted);
        assert!(result.warnings[0].contains("no attestations"));
    }

    #[test]
    fn test_no_chain_to_anchor() {
        let mut store = MemoryStore::new();

        // Random agent attests something
        store.add(Attestation::new(
            make_attestor("random", "RandomAgent"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        ));

        // But RandomAgent has no vouches and isn't an anchor
        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("trusted-id", "TrustedAgent", "the only anchor"));

        let result = validator.validate("sha256:abc123");
        assert!(!result.trusted);
    }

    #[test]
    fn test_revocation() {
        let mut store = MemoryStore::new();

        // Rufio attests weather-skill
        let attestation = Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        );
        let attestation_id = attestation.attestation_id;
        store.add(attestation);

        // Later, Rufio revokes it
        let revocation = Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::Revoke,
            Subject::skill("weather-skill", "sha256:abc123"),
        )
        .with_causation(attestation_id);
        store.add(revocation);

        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("rufio-id", "Rufio", "trusted"));

        let result = validator.validate("sha256:abc123");
        assert!(!result.trusted); // Revoked attestation shouldn't count
    }

    #[test]
    fn test_min_attestations() {
        let mut store = MemoryStore::new();

        // Only one attestation
        store.add(Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        ));

        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("rufio-id", "Rufio", "trusted"))
            .with_config(ChainConfig::default().min_attestations(2));

        let result = validator.validate("sha256:abc123");
        assert!(!result.trusted);
        assert!(result.warnings[0].contains("need 2"));
    }

    #[test]
    fn test_required_type() {
        let mut store = MemoryStore::new();

        // Only a vouch, not a security audit
        store.add(Attestation::new(
            make_attestor("rufio-id", "Rufio"),
            AttestationType::Vouch,
            Subject::skill("weather-skill", "sha256:abc123"),
        ));

        let validator = ChainValidator::new(&store)
            .add_anchor(TrustAnchor::new("rufio-id", "Rufio", "trusted"))
            .with_config(ChainConfig::default().require_security_audit());

        let result = validator.validate("sha256:abc123");
        assert!(!result.trusted);
    }

    #[test]
    fn test_cycle_detection() {
        let mut store = MemoryStore::new();

        // A attests skill
        store.add(Attestation::new(
            make_attestor("a", "AgentA"),
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        ));

        // B vouches for A
        store.add(Attestation::new(
            make_attestor("b", "AgentB"),
            AttestationType::Vouch,
            Subject::agent("AgentA", "a"),
        ));

        // A vouches for B (cycle!)
        store.add(Attestation::new(
            make_attestor("a", "AgentA"),
            AttestationType::Vouch,
            Subject::agent("AgentB", "b"),
        ));

        // No trust anchors - should detect cycle and fail
        let validator = ChainValidator::new(&store);
        let result = validator.validate("sha256:abc123");
        assert!(!result.trusted);
    }
}
