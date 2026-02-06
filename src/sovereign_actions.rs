use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Core routes through which actions travel: BCI, OTA, GOV, CHAT, RESEARCH, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SovereignRoute {
    Bci,
    Ota,
    Gov,
    Chat,
    Research,
    Other(String),
}

/// High-level action kinds that the AI layer may only *propose*.
/// Only the sovereign kernel (neuro-eXpFS + EVOLVE pipeline) can *commit*.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SovereignActionKind {
    // Neural data and neurorights
    ReadNeuralShard,
    WriteNeuralShard,
    SummarizeNeuralShard,
    // Governance and evolution
    ProposeEvolve,
    ApplyEvolve,
    // OTA / deployment
    ApplyOtaPackage,
    // Keys / signing
    ReadKeys,
    SignTransaction,
    // Generic, low-risk helper
    LowRiskHelper,
}

/// A single proposed sovereign action, before policy/tsafe evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAction {
    /// Bostrom-address or equivalent subject identifier.
    pub subject_id: String,
    /// Route: BCI, OTA, GOV, CHAT, etc.
    pub route: SovereignRoute,
    /// What type of action is being requested.
    pub kind: SovereignActionKind,
    /// Context labels for policy / RoH evaluation (e.g., "dreamstatesensitive").
    pub context_labels: Vec<String>,
    /// Fine-grain fields the caller is asking for (e.g., "summary", "metadata").
    pub requested_fields: Vec<String>,
    /// Estimated lifeforce / bioscale cost of the action.
    pub lifeforce_cost: f32,
    /// Optional ALN shard IDs or logical references (never raw paths).
    pub shard_refs: Vec<String>,
}

/// Decision outcome from Tsafe Cortex Gate / PolicyEngine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignDecision {
    Allow {
        reason: String,
    },
    AllowWithConstraints {
        reason: String,
        // Fields that must be redacted or withheld.
        redactions: Vec<String>,
    },
    Deny {
        reason: String,
    },
}

/// A strongly-typed capability that the LLM can *use* but never *forge*.
/// In practice, construction should happen only inside trusted Rust code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    /// Summarize already-redacted text about neurorights or policies.
    SummarizeText,
    /// Generate Rust helper code that never runs without human review.
    GenerateRustHelper,
    /// Explain policy/ALN shape in human terms.
    ExplainPolicyShape,
    /// Draft an EVOLVE proposal (never apply it directly).
    DraftEvolveProposal,
    /// Generic safe explanation / QA.
    ExplainConcept,
}

/// A short-lived, scoped capability “chord” the LLM can be bound to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    /// Opaque ID, to be mapped internally to logs and Donutloop entries.
    pub id: uuid::Uuid,
    /// What the capability allows, at a high level.
    pub kind: CapabilityKind,
    /// Subject/Bostrom ID this capability is bound to.
    pub subject_id: String,
    /// Route on which this capability is valid.
    pub route: SovereignRoute,
    /// Maximum LLM tokens this capability may consume (budgeting, RoH).
    pub max_tokens: u32,
    /// Hard expiry time for this capability.
    pub expires_at: SystemTime,
}

impl CapabilityChord {
    /// Construct a new capability with a default short lifetime.
    pub fn new(kind: CapabilityKind, subject_id: impl Into<String>, route: SovereignRoute) -> Self {
        let now = SystemTime::now();
        let ttl = Duration::from_secs(60); // 60s lifetime; tune via .tsafe.aln
        Self {
            id: uuid::Uuid::new_v4(),
            kind,
            subject_id: subject_id.into(),
            route,
            max_tokens: 1024,
            expires_at: now + ttl,
        }
    }

    /// Check if the capability is expired.
    pub fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.expires_at)
            .is_ok()
    }
}

/// Minimal policy surface for Tsafe Cortex Gate.
/// In a full implementation, this would load .neurorights.json, .tsafe.aln, .vkernel.aln.
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn new() -> Self {
        PolicyEngine
    }

    /// Evaluate a proposed action against compiled neurorights / tsafe policy.
    /// Here we only encode the *shape*; the real logic will consult ALN shards.
    pub fn evaluate(&self, action: &SovereignAction) => SovereignDecision {
        // Example: hard block raw neural shard reads over AI-mediated routes.
        if matches!(action.kind, SovereignActionKind::ReadNeuralShard) {
            return SovereignDecision::Deny {
                reason: "Mental privacy: raw neural shard reads are forbidden over AI routes".into(),
            };
        }

        // Example: block key reads entirely over AI routes.
        if matches!(action.kind, SovereignActionKind::ReadKeys) {
            return SovereignDecision::Deny {
                reason: "Keys may not be read via AI-mediated routes".into(),
            };
        }

        // Example: constrain dreamstate-sensitive fields.
        let mut redactions = Vec::new();
        if action
            .context_labels
            .iter()
            .any(|l| l == "dreamstatesensitive")
        {
            redactions.push("dream_segments".to_string());
        }

        if redactions.is_empty() {
            SovereignDecision::Allow {
                reason: "Allowed by Tsafe / neurorights constraints".into(),
            }
        } else {
            SovereignDecision::AllowWithConstraints {
                reason: "Allowed with dreamstate redactions".into(),
                redactions,
            }
        }
    }
}
