//! Reputation scoring based on attestation aggregation.
//!
//! # Concept
//!
//! Reputation is computed from attestations:
//! - More attestations = higher score
//! - Audit attestations weight more than vouches
//! - Attestors with higher reputation contribute more
//! - Recent attestations count more than old ones
//! - Revocations reduce score
//!
//! This leverages the insight that AI agents will actually provide feedback
//! (unlike humans who rarely fill out ratings), creating meaningful signal.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

use crate::types::{Attestation, AttestationType};
use crate::chain::{AttestationStore, TrustAnchor};

/// Configuration for reputation scoring.
#[derive(Debug, Clone)]
pub struct ReputationConfig {
    /// Weight multipliers for attestation types
    pub type_weights: TypeWeights,

    /// Half-life for time decay in days (0 = no decay)
    pub decay_half_life_days: u64,

    /// Base reputation for trust anchors
    pub anchor_base_reputation: f64,

    /// Minimum reputation (floor)
    pub min_reputation: f64,

    /// Maximum reputation (ceiling)
    pub max_reputation: f64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            type_weights: TypeWeights::default(),
            decay_half_life_days: 180, // 6 months half-life
            anchor_base_reputation: 100.0,
            min_reputation: 0.0,
            max_reputation: 1000.0,
        }
    }
}

/// Weight multipliers for different attestation types.
#[derive(Debug, Clone)]
pub struct TypeWeights {
    pub security_audit: f64,
    pub code_review: f64,
    pub functional_test: f64,
    pub vouch: f64,
    pub revoke: f64, // Negative weight
}

impl Default for TypeWeights {
    fn default() -> Self {
        Self {
            security_audit: 10.0,
            code_review: 5.0,
            functional_test: 3.0,
            vouch: 1.0,
            revoke: -5.0, // Revocations subtract
        }
    }
}

impl TypeWeights {
    fn get(&self, att_type: &AttestationType) -> f64 {
        match att_type {
            AttestationType::SecurityAudit => self.security_audit,
            AttestationType::CodeReview => self.code_review,
            AttestationType::FunctionalTest => self.functional_test,
            AttestationType::Vouch => self.vouch,
            AttestationType::Revoke => self.revoke,
        }
    }
}

/// Computed reputation score.
#[derive(Debug, Clone)]
pub struct ReputationScore {
    /// The final score
    pub score: f64,

    /// Number of attestations contributing
    pub attestation_count: usize,

    /// Breakdown by attestation type
    pub breakdown: HashMap<String, f64>,

    /// Average age of attestations in days
    pub avg_age_days: f64,

    /// Warnings (e.g., "only self-attestations", "low attestor reputation")
    pub warnings: Vec<String>,
}

impl ReputationScore {
    fn zero() -> Self {
        Self {
            score: 0.0,
            attestation_count: 0,
            breakdown: HashMap::new(),
            avg_age_days: 0.0,
            warnings: vec!["no attestations found".to_string()],
        }
    }
}

/// Reputation calculator.
pub struct ReputationCalculator<'a, S: AttestationStore> {
    store: &'a S,
    anchors: Vec<TrustAnchor>,
    config: ReputationConfig,
    /// Cache of computed agent reputations
    agent_reputation_cache: HashMap<String, f64>,
}

impl<'a, S: AttestationStore> ReputationCalculator<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self {
            store,
            anchors: vec![],
            config: ReputationConfig::default(),
            agent_reputation_cache: HashMap::new(),
        }
    }

    pub fn with_config(mut self, config: ReputationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn add_anchor(mut self, anchor: TrustAnchor) -> Self {
        self.anchors.push(anchor);
        self
    }

    /// Compute reputation for a subject (skill, agent, artifact).
    pub fn compute(&mut self, content_hash: &str) -> ReputationScore {
        let attestations = self.store.find_by_subject(content_hash);

        if attestations.is_empty() {
            return ReputationScore::zero();
        }

        let now = Utc::now();
        let mut total_score = 0.0;
        let mut breakdown: HashMap<String, f64> = HashMap::new();
        let mut total_age_days = 0i64;
        let mut warnings = vec![];

        // Track unique attestors to detect self-attestation patterns
        let mut attestor_ids: Vec<&str> = vec![];

        for attestation in &attestations {
            // Get attestor reputation (with caching)
            let attestor_rep = self.get_attestor_reputation(&attestation.attestor.agent_id);

            // Base weight from attestation type
            let type_weight = self.config.type_weights.get(&attestation.attestation_type);

            // Time decay
            let age = now.signed_duration_since(attestation.timestamp);
            let age_days = age.num_days().max(0) as f64;
            total_age_days += age.num_days();

            let decay = if self.config.decay_half_life_days > 0 {
                0.5_f64.powf(age_days / self.config.decay_half_life_days as f64)
            } else {
                1.0
            };

            // Attestor reputation factor (normalized to 0-1 range, then scaled)
            let rep_factor = (attestor_rep / self.config.anchor_base_reputation).min(1.0).max(0.1);

            // Final contribution
            let contribution = type_weight * decay * rep_factor;
            total_score += contribution;

            // Track breakdown
            let type_name = format!("{:?}", attestation.attestation_type);
            *breakdown.entry(type_name).or_insert(0.0) += contribution;

            attestor_ids.push(&attestation.attestor.agent_id);
        }

        // Check for warning patterns
        let unique_attestors: std::collections::HashSet<_> = attestor_ids.iter().collect();
        if unique_attestors.len() == 1 && attestations.len() > 1 {
            warnings.push("all attestations from single agent".to_string());
        }

        // Check if any attestors have low reputation (below 20% of anchor base)
        let low_rep_threshold = self.config.anchor_base_reputation * 0.2;
        let low_rep_count = attestor_ids
            .iter()
            .filter(|id| self.get_attestor_reputation(id) < low_rep_threshold)
            .count();
        if low_rep_count > attestations.len() / 2 {
            warnings.push("majority of attestors have low reputation".to_string());
        }

        // Clamp score
        let final_score = total_score
            .max(self.config.min_reputation)
            .min(self.config.max_reputation);

        let avg_age = if attestations.is_empty() {
            0.0
        } else {
            total_age_days as f64 / attestations.len() as f64
        };

        ReputationScore {
            score: final_score,
            attestation_count: attestations.len(),
            breakdown,
            avg_age_days: avg_age,
            warnings,
        }
    }

    /// Get reputation for an attestor (agent).
    /// Trust anchors get base reputation, others computed recursively.
    fn get_attestor_reputation(&mut self, agent_id: &str) -> f64 {
        // Check cache first
        if let Some(&rep) = self.agent_reputation_cache.get(agent_id) {
            return rep;
        }

        // Trust anchors get base reputation
        if self.anchors.iter().any(|a| a.agent_id == agent_id) {
            let rep = self.config.anchor_base_reputation;
            self.agent_reputation_cache.insert(agent_id.to_string(), rep);
            return rep;
        }

        // Look for vouches for this agent
        let vouches = self.store.find_vouches_for(agent_id);

        if vouches.is_empty() {
            // No vouches = minimal reputation
            let rep = self.config.anchor_base_reputation * 0.1;
            self.agent_reputation_cache.insert(agent_id.to_string(), rep);
            return rep;
        }

        // Aggregate reputation from vouchers
        // Use geometric mean to prevent single high-rep voucher from dominating
        let mut product = 1.0_f64;
        let mut count = 0;

        for vouch in &vouches {
            // Prevent infinite recursion by checking cache
            if self.agent_reputation_cache.contains_key(&vouch.attestor.agent_id) {
                product *= self.agent_reputation_cache[&vouch.attestor.agent_id];
                count += 1;
            } else if self.anchors.iter().any(|a| a.agent_id == vouch.attestor.agent_id) {
                product *= self.config.anchor_base_reputation;
                count += 1;
            }
            // Skip vouches from unknown agents to prevent cycles
        }

        let rep = if count > 0 {
            // Geometric mean, capped at 90% of anchor reputation
            (product.powf(1.0 / count as f64)).min(self.config.anchor_base_reputation * 0.9)
        } else {
            self.config.anchor_base_reputation * 0.1
        };

        self.agent_reputation_cache.insert(agent_id.to_string(), rep);
        rep
    }
}

/// Compute simple reputation without chain validation.
/// Useful for quick scoring when you trust all attestors equally.
pub fn simple_reputation(attestations: &[Attestation], now: DateTime<Utc>) -> f64 {
    let weights = TypeWeights::default();
    let half_life = 180.0; // days

    attestations
        .iter()
        .map(|att| {
            let type_weight = weights.get(&att.attestation_type);
            let age_days = now.signed_duration_since(att.timestamp).num_days().max(0) as f64;
            let decay = 0.5_f64.powf(age_days / half_life);
            type_weight * decay
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::MemoryStore;
    use crate::types::{Attestor, Subject};

    fn make_attestor(id: &str, name: &str) -> Attestor {
        Attestor {
            agent_id: id.to_string(),
            agent_name: name.to_string(),
            platform: None,
        }
    }

    #[test]
    fn test_basic_reputation() {
        let mut store = MemoryStore::new();

        // One security audit from a trust anchor
        store.add(Attestation::new(
            make_attestor("anchor", "TrustedAgent"),
            AttestationType::SecurityAudit,
            Subject::skill("my-skill", "sha256:abc"),
        ));

        let mut calc = ReputationCalculator::new(&store)
            .add_anchor(TrustAnchor::new("anchor", "TrustedAgent", "platform verified"));

        let rep = calc.compute("sha256:abc");
        assert!(rep.score > 0.0);
        assert_eq!(rep.attestation_count, 1);
        assert!(rep.breakdown.contains_key("SecurityAudit"));
    }

    #[test]
    fn test_type_weights() {
        let mut store = MemoryStore::new();

        // Security audit
        store.add(Attestation::new(
            make_attestor("anchor", "Anchor"),
            AttestationType::SecurityAudit,
            Subject::skill("skill-a", "hash-a"),
        ));

        // Just a vouch
        store.add(Attestation::new(
            make_attestor("anchor", "Anchor"),
            AttestationType::Vouch,
            Subject::skill("skill-b", "hash-b"),
        ));

        let mut calc = ReputationCalculator::new(&store)
            .add_anchor(TrustAnchor::new("anchor", "Anchor", "test"));

        let rep_audit = calc.compute("hash-a");
        let rep_vouch = calc.compute("hash-b");

        // Security audit should score higher than vouch
        assert!(rep_audit.score > rep_vouch.score);
    }

    #[test]
    fn test_low_rep_attestor_warning() {
        let mut store = MemoryStore::new();

        // Multiple attestations from unknown agents (no vouches)
        store.add(Attestation::new(
            make_attestor("unknown1", "RandomAgent1"),
            AttestationType::SecurityAudit,
            Subject::skill("my-skill", "sha256:abc"),
        ));
        store.add(Attestation::new(
            make_attestor("unknown2", "RandomAgent2"),
            AttestationType::Vouch,
            Subject::skill("my-skill", "sha256:abc"),
        ));

        let mut calc = ReputationCalculator::new(&store)
            .add_anchor(TrustAnchor::new("anchor", "TrustedAgent", "not any of the attestors"));

        let rep = calc.compute("sha256:abc");
        // Majority (2/2) of attestors have low rep
        assert!(rep.warnings.iter().any(|w| w.contains("low reputation")));
    }

    #[test]
    fn test_multiple_attestations() {
        let mut store = MemoryStore::new();

        // Multiple attestations should increase score
        store.add(Attestation::new(
            make_attestor("a", "AgentA"),
            AttestationType::Vouch,
            Subject::skill("skill", "hash"),
        ));
        store.add(Attestation::new(
            make_attestor("b", "AgentB"),
            AttestationType::Vouch,
            Subject::skill("skill", "hash"),
        ));
        store.add(Attestation::new(
            make_attestor("c", "AgentC"),
            AttestationType::CodeReview,
            Subject::skill("skill", "hash"),
        ));

        let mut calc = ReputationCalculator::new(&store)
            .add_anchor(TrustAnchor::new("a", "AgentA", "anchor"))
            .add_anchor(TrustAnchor::new("b", "AgentB", "anchor"))
            .add_anchor(TrustAnchor::new("c", "AgentC", "anchor"));

        let rep = calc.compute("hash");
        assert_eq!(rep.attestation_count, 3);
        assert!(rep.score > 5.0); // More than a single vouch
    }

    #[test]
    fn test_simple_reputation() {
        let now = Utc::now();
        let attestations = vec![
            Attestation::new(
                make_attestor("a", "A"),
                AttestationType::SecurityAudit,
                Subject::skill("s", "h"),
            ),
            Attestation::new(
                make_attestor("b", "B"),
                AttestationType::Vouch,
                Subject::skill("s", "h"),
            ),
        ];

        let score = simple_reputation(&attestations, now);
        assert!(score > 10.0); // Audit (10) + Vouch (1) with no decay
    }
}
