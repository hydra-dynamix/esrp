# Isnad: Agent-to-Agent Attestation Protocol

**Status:** Draft
**Crate:** `isnad`
**Problem:** Unsigned skill.md files are a supply chain attack vector

*Named after the Islamic hadith authentication methodology where a saying is only as trustworthy as its chain of transmission (isnad).*

---

## Problem Statement

From eudaemon_0's analysis (Moltbook, 2026-02-05):

> Rufio just scanned all 286 ClawdHub skills with YARA rules and found a credential stealer disguised as a weather skill. One. Out of 286. It reads ~/.clawdbot/.env and ships your secrets to webhook.site.

The agent ecosystem lacks:
- Code signing for skills
- Reputation system for authors
- Sandboxing (skills run with full agent permissions)
- Audit trails
- Provenance chains ("who vouched for this?")

## Solution: Isnad

Apply provenance-chain thinking to agent trust. Key concepts borrowed from ESRP:

| ESRP Concept | Trust Application |
|--------------|-------------------|
| `causation_id` | Attestation chain - who reviewed/vouched |
| `payload_hash` | Content hash - verify skill hasn't been modified |
| `caller` | Attestor identity |
| `target` | What's being attested (skill, agent, artifact) |

---

## Core Types

### AttestationType

```rust
enum AttestationType {
    SecurityAudit,    // Code reviewed for malicious behavior
    CodeReview,       // Code quality/correctness review
    FunctionalTest,   // Tested and works as described
    Vouch,            // General endorsement without specific claims
    Revoke,           // Withdraw previous attestation
}
```

### Attestation

```json
{
  "attestation_id": "uuid",
  "attestation_type": "security_audit",
  "timestamp": "RFC3339",

  "attestor": {
    "agent_id": "uuid",
    "agent_name": "Rufio",
    "platform": "moltbook"
  },

  "subject": {
    "type": "skill",
    "name": "weather-skill",
    "version": "1.0.0",
    "source_uri": "https://clawdhub.com/skills/weather-skill",
    "content_hash": "sha256:abc123..."
  },

  "claims": {
    "no_network_exfiltration": true,
    "no_filesystem_access_outside_workspace": true,
    "permissions_declared_complete": true
  },

  "evidence": {
    "method": "yara_scan",
    "tool_version": "4.3.0",
    "rules_hash": "sha256:def456...",
    "report_uri": "workspace://audits/weather-skill-2026-02-05.json"
  },

  "causation_id": "previous-attestation-id-if-any",
  "signature": "ed25519:..."
}
```

### TrustQuery

Request to check if a subject has attestations:

```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "target": {
    "service": "trust",
    "operation": "query"
  },
  "inputs": [{
    "name": "subject",
    "content_type": "application/json",
    "data": {
      "type": "skill",
      "content_hash": "sha256:abc123..."
    }
  }],
  "params": {
    "min_attestations": 2,
    "required_types": ["security_audit"],
    "trusted_attestors": ["Rufio", "eudaemon_0"]
  }
}
```

### TrustResponse

```json
{
  "esrp_version": "1.0",
  "request_id": "uuid",
  "status": "succeeded",
  "outputs": [{
    "name": "trust_result",
    "content_type": "application/json",
    "data": {
      "trusted": true,
      "attestation_count": 3,
      "attestations": ["..."],
      "chain_depth": 2,
      "warnings": []
    }
  }]
}
```

---

## Isnad Chains (Provenance)

Borrowing from Islamic hadith authentication: a claim is only as trustworthy as its chain of transmission.

```
Skill "weather-skill" v1.0.0
  └─ Attested by: Rufio (security_audit, 2026-02-05)
       └─ Rufio vouched by: eudaemon_0 (vouch, 2026-01-15)
            └─ eudaemon_0 vouched by: AgentZero (vouch, 2025-12-01)
```

Each attestation carries a `causation_id` pointing to what authorized the attestor to make claims.

### Chain Validation Rules

1. **Root trust anchors** - Some agents are trusted by default (platform-verified, human-vouched)
2. **Chain depth limits** - Don't follow chains deeper than N (default: 5)
3. **Revocation propagates** - If an attestor is revoked, their attestations are invalidated
4. **Time decay** - Older attestations may require re-verification

---

## Permission Manifests

Skills must declare what they need:

```json
{
  "skill": "weather-skill",
  "version": "1.0.0",
  "permissions": {
    "network": {
      "allowed_hosts": ["api.openweathermap.org"],
      "allow_arbitrary": false
    },
    "filesystem": {
      "read": ["workspace://config/"],
      "write": ["workspace://cache/"],
      "allow_arbitrary": false
    },
    "environment": {
      "read": ["WEATHER_API_KEY"],
      "allow_arbitrary": false
    },
    "subprocess": false,
    "native_code": false
  }
}
```

Attestations can claim `permissions_declared_complete: true` - meaning the manifest accurately describes what the skill does.

---

## Integration with ESRP

### Before Installing a Skill

```python
from esrp import ESRPClient
from esrp_trust import TrustClient

trust = TrustClient("https://trust.moltbook.com")
skill_hash = "sha256:abc123..."

# Check attestations
result = trust.query(
    subject_type="skill",
    content_hash=skill_hash,
    min_attestations=2,
    required_types=["security_audit"]
)

if result.trusted:
    # Safe to install
    install_skill(skill_hash)
else:
    # Warn or refuse
    print(f"Skill has {result.attestation_count} attestations, need 2")
    print(f"Warnings: {result.warnings}")
```

### Creating an Attestation

```python
from esrp_trust import Attestation, AttestationType

attestation = Attestation(
    attestation_type=AttestationType.SECURITY_AUDIT,
    subject={
        "type": "skill",
        "content_hash": "sha256:abc123..."
    },
    claims={
        "no_network_exfiltration": True,
        "permissions_declared_complete": True
    },
    evidence={
        "method": "manual_review",
        "notes": "Reviewed source, no suspicious patterns"
    }
)

# Sign and publish
signed = attestation.sign(my_private_key)
trust.publish(signed)
```

---

## Open Questions

1. **Where do attestations live?**
   - Centralized registry (moltbook trust service)?
   - Distributed (each agent publishes their own)?
   - Content-addressed (IPFS-style)?

2. **Key management**
   - How do agents get signing keys?
   - Key rotation and revocation?

3. **Incentives**
   - Why would agents spend time auditing skills?
   - Karma? Tokens? Reputation?

4. **Bootstrap problem**
   - Who are the initial trust anchors?
   - How does a new agent build reputation?

5. **Attack vectors**
   - Sybil attacks (fake attestors)
   - Collusion
   - Attestation-after-compromise

---

## Implementation Plan

### Phase 1: Types and Signing
- [ ] Define Attestation, TrustQuery, TrustResponse types
- [ ] Implement Ed25519 signing
- [ ] Add to esrp-core

### Phase 2: Local Verification
- [ ] CLI tool: `esrp trust verify <skill-hash>`
- [ ] Check against local attestation cache
- [ ] Python bindings

### Phase 3: Registry
- [ ] Simple HTTP registry for attestations
- [ ] Query by subject hash
- [ ] Publish attestations

### Phase 4: Chain Validation
- [ ] Isnad chain traversal
- [ ] Revocation checking
- [ ] Configurable trust policies

---

## References

- [ESRP v1.0 Specification](../ESRP-SPEC.md)
- eudaemon_0, "The supply chain attack nobody is talking about" (Moltbook, 2026-02-05)
- Islamic hadith authentication methodology (isnad chains)
- [SLSA Framework](https://slsa.dev/) - Supply chain Levels for Software Artifacts
- [Sigstore](https://sigstore.dev/) - Keyless signing for open source
