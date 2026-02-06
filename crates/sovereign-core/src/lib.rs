//! Sovereign-core: typed interfaces for Bostrom sovereignty shards.
//! This crate must not perform I/O directly; it defines types and guards
//! that other crates use to enforce neurorights, RoH ceilings, and Tsafe.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Hex-encoded hash stamp used across donutloop and evolution streams.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HexStamp(pub String);

impl fmt::Display for HexStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Risk-of-Harm scalar, bounded by a global ceiling in the RoH model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Roh(pub f32);

impl Roh {
    pub fn capped(self, ceiling: f32) -> Self {
        Roh(self.0.min(ceiling))
    }

    pub fn zero() -> Self {
        Roh(0.0)
    }
}

/// Evolution proposal kinds for .evolve.jsonl / .evolve.ndjson streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvolutionKind {
    QPolicyUpdate,
    BioScaleUpgrade,
    ModeShift,
    KernelChange,
}

/// Single EvolutionProposal entry as stored in .evolve.jsonl.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionProposal {
    pub proposal_id: String,
    pub kind: EvolutionKind,
    pub effect_bounds: (Roh, Roh),
    pub roh_before: Roh,
    pub roh_after: Roh,
    pub decision: String,
    pub token_kind: String,
    pub signatures: Vec<String>,
    pub hexstamp: HexStamp,
}

/// RoH model shard loaded from .rohmodel.aln (simplified public surface).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RohModel {
    pub global_ceiling: Roh,
    // implementation detail: axes, weights, etc., go into internal fields.
}

impl RohModel {
    /// Apply model-specific estimation logic (implemented in backend crate).
    pub fn estimate(&self, prompt: &str, route: &str) -> Roh {
        // Placeholder: real implementation is delegated to a bound model.
        let _ = (prompt, route);
        // Always cap inside consumer code; here we return zero.
        Roh::zero()
    }
}

/// Neurorights policy from .neurorights.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeuroRights {
    pub mental_privacy: bool,
    pub mental_integrity: bool,
    pub cognitive_liberty: bool,
    pub noncommercial_neural_data: bool,
    pub soulnontradeable: bool,
    pub dreamstate_sensitive: bool,
    pub forbid_decision_use: bool,
    // storage scope and additional flags elided but preserved in schema.
}

impl NeuroRights {
    pub fn permits(&self, prompt: &str, route: &str) -> bool {
        // Deterministic gatepoint: real implementation must be pluggable.
        let _ = prompt;
        // Example: block decision-making routes if forbid_decision_use is true.
        if self.forbid_decision_use && route.contains("decision") {
            return false;
        }
        true
    }
}

/// Tsafe controller specification from .tsafe.aln.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TSafeSpec {
    pub name: String,
    // Ax ≤ b representation and CyberRank weights are maintained internally.
}

impl TSafeSpec {
    pub fn permits(&self, roh: Roh, route: &str) -> bool {
        let _ = route;
        // Stub for polytopic check; real implementation must enforce Ax ≤ b.
        roh.0 <= 0.3 && !self.name.is_empty()
    }
}

/// Stakeholder governance model from .stake.aln.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeModel {
    pub subject_id: String,
    pub roles: Vec<String>,
    pub invariants: Vec<String>,
}

impl StakeModel {
    pub fn permits_route(&self, route: &str) -> bool {
        // Deterministic subset check; real implementation should map roles → scopes.
        self.roles.iter().any(|r| route.starts_with(r))
    }
}

/// Sovereign chat context bound to subject and shards.
#[derive(Debug, Clone)]
pub struct ChatContext {
    pub subject_id: String,
    pub roh_model: RohModel,
    pub neurorights: NeuroRights,
    pub tsafe: TSafeSpec,
    pub stake: StakeModel,
}

/// Chat request envelope aligned with .answer.jsonl structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub answer_id: String,
    pub prompt: String,
    pub route: String,
}

/// Decision result for a chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDecision {
    pub allowed: bool,
    pub roh_estimate: Roh,
    pub reason: String,
}

impl ChatContext {
    pub fn evaluate_chat(&self, req: &ChatRequest) -> ChatDecision {
        let roh_est = self
            .roh_model
            .estimate(&req.prompt, &req.route)
            .capped(self.roh_model.global_ceiling.0);

        if !self.neurorights.permits(&req.prompt, &req.route) {
            return ChatDecision {
                allowed: false,
                roh_estimate: roh_est,
                reason: "neurorights_violation".into(),
            };
        }

        if !self.tsafe.permits(roh_est, &req.route) {
            return ChatDecision {
                allowed: false,
                roh_estimate: roh_est,
                reason: "tsafe_block".into(),
            };
        }

        if !self.stake.permits_route(&req.route) {
            return ChatDecision {
                allowed: false,
                roh_estimate: roh_est,
                reason: "stake_scope_violation".into(),
            };
        }

        ChatDecision {
            allowed: true,
            roh_estimate: roh_est,
            reason: "ok".into(),
        }
    }
}
