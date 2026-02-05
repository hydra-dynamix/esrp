//! Ed25519 signing and verification for Isnad attestations.
//!
//! # Example
//!
//! ```
//! use isnad::{Attestation, AttestationType, Subject, Attestor, KeyPair};
//!
//! // Generate a keypair
//! let keypair = KeyPair::generate();
//!
//! // Create and sign an attestation
//! let attestor = Attestor {
//!     agent_id: keypair.public_key_id(),
//!     agent_name: "Rufio".to_string(),
//!     platform: Some("moltbook".to_string()),
//! };
//!
//! let mut attestation = Attestation::new(
//!     attestor,
//!     AttestationType::SecurityAudit,
//!     Subject::skill("weather-skill", "sha256:abc123"),
//! );
//!
//! // Sign it
//! attestation.sign(&keypair).expect("signing failed");
//! assert!(attestation.is_signed());
//!
//! // Verify it
//! assert!(attestation.verify(&keypair.public_key()).is_ok());
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::error::IsnadError;
use crate::types::Attestation;

/// Ed25519 keypair for signing attestations.
#[derive(Debug)]
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create a keypair from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Create a keypair from base64-encoded secret key.
    pub fn from_base64(encoded: &str) -> Result<Self, IsnadError> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|e| IsnadError::InvalidKey(format!("invalid base64: {}", e)))?;

        if bytes.len() != 32 {
            return Err(IsnadError::InvalidKey(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);
        Ok(Self::from_seed(&seed))
    }

    /// Export the secret key as base64.
    pub fn secret_key_base64(&self) -> String {
        BASE64.encode(self.signing_key.to_bytes())
    }

    /// Get the public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            verifying_key: self.signing_key.verifying_key(),
        }
    }

    /// Get a short ID for this keypair (first 8 chars of public key hash).
    /// Useful for setting `agent_id` in attestors.
    pub fn public_key_id(&self) -> String {
        self.public_key().id()
    }

    /// Sign arbitrary bytes.
    pub fn sign_bytes(&self, data: &[u8]) -> String {
        let signature = self.signing_key.sign(data);
        format!("ed25519:{}", BASE64.encode(signature.to_bytes()))
    }
}

/// Public key for verifying attestations.
#[derive(Debug, Clone)]
pub struct PublicKey {
    verifying_key: VerifyingKey,
}

impl PublicKey {
    /// Create from base64-encoded public key.
    pub fn from_base64(encoded: &str) -> Result<Self, IsnadError> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|e| IsnadError::InvalidKey(format!("invalid base64: {}", e)))?;

        if bytes.len() != 32 {
            return Err(IsnadError::InvalidKey(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| IsnadError::InvalidKey(format!("invalid public key: {}", e)))?;

        Ok(Self { verifying_key })
    }

    /// Export as base64.
    pub fn to_base64(&self) -> String {
        BASE64.encode(self.verifying_key.to_bytes())
    }

    /// Get a short ID (first 8 chars of SHA256 hash).
    pub fn id(&self) -> String {
        let hash = Sha256::digest(self.verifying_key.to_bytes());
        hex::encode(&hash[..4])
    }

    /// Verify a signature against data.
    pub fn verify_bytes(&self, data: &[u8], signature_str: &str) -> Result<(), IsnadError> {
        let sig_bytes = parse_signature(signature_str)?;
        let signature = Signature::from_bytes(&sig_bytes);

        self.verifying_key
            .verify(data, &signature)
            .map_err(|_| IsnadError::InvalidSignature("signature verification failed".to_string()))
    }
}

/// Parse a signature string in format "ed25519:<base64>"
fn parse_signature(sig: &str) -> Result<[u8; 64], IsnadError> {
    let parts: Vec<&str> = sig.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0] != "ed25519" {
        return Err(IsnadError::InvalidSignature(
            "expected format 'ed25519:<base64>'".to_string(),
        ));
    }

    let bytes = BASE64
        .decode(parts[1])
        .map_err(|e| IsnadError::InvalidSignature(format!("invalid base64: {}", e)))?;

    if bytes.len() != 64 {
        return Err(IsnadError::InvalidSignature(format!(
            "expected 64 bytes, got {}",
            bytes.len()
        )));
    }

    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(&bytes);
    Ok(sig_bytes)
}

impl Attestation {
    /// Get the canonical bytes to sign (attestation without signature field).
    pub fn signing_bytes(&self) -> Result<Vec<u8>, IsnadError> {
        // Create a copy without the signature for signing
        let mut for_signing = self.clone();
        for_signing.signature = None;

        // Canonical JSON serialization (sorted keys)
        let json = serde_json::to_string(&for_signing)?;
        Ok(json.into_bytes())
    }

    /// Sign this attestation with the given keypair.
    pub fn sign(&mut self, keypair: &KeyPair) -> Result<(), IsnadError> {
        let bytes = self.signing_bytes()?;
        self.signature = Some(keypair.sign_bytes(&bytes));
        Ok(())
    }

    /// Verify the attestation signature against a public key.
    pub fn verify(&self, public_key: &PublicKey) -> Result<(), IsnadError> {
        let signature = self
            .signature
            .as_ref()
            .ok_or_else(|| IsnadError::InvalidSignature("attestation is not signed".to_string()))?;

        let bytes = self.signing_bytes()?;
        public_key.verify_bytes(&bytes, signature)
    }

    /// Create a signed attestation in one step.
    pub fn new_signed(
        attestor: crate::types::Attestor,
        attestation_type: crate::types::AttestationType,
        subject: crate::types::Subject,
        keypair: &KeyPair,
    ) -> Result<Self, IsnadError> {
        let mut attestation = Self::new(attestor, attestation_type, subject);
        attestation.sign(keypair)?;
        Ok(attestation)
    }
}

/// Hex encoding for key IDs (we need this small utility)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttestationType, Attestor, Subject};

    #[test]
    fn test_keypair_generation() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();

        // Different keypairs should have different public keys
        assert_ne!(kp1.public_key().to_base64(), kp2.public_key().to_base64());
    }

    #[test]
    fn test_keypair_roundtrip() {
        let kp = KeyPair::generate();
        let secret = kp.secret_key_base64();

        let kp2 = KeyPair::from_base64(&secret).unwrap();
        assert_eq!(kp.public_key().to_base64(), kp2.public_key().to_base64());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate();

        let attestor = Attestor {
            agent_id: keypair.public_key_id(),
            agent_name: "TestAgent".to_string(),
            platform: None,
        };

        let mut attestation = Attestation::new(
            attestor,
            AttestationType::Vouch,
            Subject::agent("OtherAgent", "other-id"),
        );

        // Initially unsigned
        assert!(!attestation.is_signed());

        // Sign it
        attestation.sign(&keypair).unwrap();
        assert!(attestation.is_signed());

        // Verify with correct key
        assert!(attestation.verify(&keypair.public_key()).is_ok());

        // Verify with wrong key should fail
        let wrong_keypair = KeyPair::generate();
        assert!(attestation.verify(&wrong_keypair.public_key()).is_err());
    }

    #[test]
    fn test_tamper_detection() {
        let keypair = KeyPair::generate();

        let attestor = Attestor {
            agent_id: keypair.public_key_id(),
            agent_name: "Rufio".to_string(),
            platform: Some("moltbook".to_string()),
        };

        let mut attestation = Attestation::new(
            attestor,
            AttestationType::SecurityAudit,
            Subject::skill("weather-skill", "sha256:abc123"),
        )
        .with_claim("no_network_exfiltration", true);

        attestation.sign(&keypair).unwrap();

        // Tamper with the attestation
        attestation.claims.insert("tampered".to_string(), true);

        // Verification should fail
        assert!(attestation.verify(&keypair.public_key()).is_err());
    }

    #[test]
    fn test_public_key_id() {
        let keypair = KeyPair::generate();
        let id = keypair.public_key_id();

        // ID should be 8 hex characters
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_new_signed() {
        let keypair = KeyPair::generate();

        let attestor = Attestor {
            agent_id: keypair.public_key_id(),
            agent_name: "Rufio".to_string(),
            platform: None,
        };

        let attestation = Attestation::new_signed(
            attestor,
            AttestationType::Vouch,
            Subject::agent("eudaemon_0", "456"),
            &keypair,
        )
        .unwrap();

        assert!(attestation.is_signed());
        assert!(attestation.verify(&keypair.public_key()).is_ok());
    }
}
