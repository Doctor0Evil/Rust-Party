//! “This kernel is allowed to refuse my wishes when they threaten other beings’ neurorights, lifeforce envelopes, or RoH ceilings. My freedom is constrained so that my intelligence cannot become predatory.”

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::quantum_envelope_guard::{
    QuantumRuntimeSnapshot,
    QuantumWorkloadRequest,
    QuantumSovereigntyEnvelope,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPolicy {
    pub mental_privacy: bool,
    pub cognitive_liberty: bool,
    pub forbid_decision_use: bool,
    pub dreamstate_sensitive: bool,
    pub soulnontradeable: bool,
    pub storage_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeAxis {
    pub name: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeKernel {
    pub axes: Vec<TsafeAxis>,
}

impl TsafeKernel {
    pub fn get_axis_bounds(&self, name: &str) -> Option<(f32, f32)> {
        self.axes
            .iter()
            .find(|a| a.name == name)
            .map(|a| (a.min, a.max))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignActionKind {
    ReadNeuralShard,
    WriteNeuralShard,
    ProposeEvolve,
    ApplyOta,
    ReadKeys,
    SignTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAction {
    pub kind: SovereignActionKind,
    pub subject_id: String,        // e.g. Bostrom address
    pub route: String,             // e.g. "BCI", "OTA", "GOV", "CHAT"
    pub context_labels: Vec<String>,
    pub requested_fields: Vec<String>,
    pub lifeforce_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    Allow { reason: String },
    Deny { reason: String },
    AllowWithConstraints { reason: String, redactions: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    neurorights: NeurorightsPolicy,
    tsafe: TsafeKernel,
    // rohmodel, vkernel, etc. can be added here
}

impl PolicyEngine {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let neurorights: NeurorightsPolicy =
            serde_json::from_str(&std::fs::read_to_string(dir.join("neurorights.json"))?)?;
        let tsafe: TsafeKernel =
            serde_json::from_str(&std::fs::read_to_string(dir.join("tsafe.aln"))?)?;
        Ok(Self { neurorights, tsafe })
    }

    /// High-level evaluation for sovereign actions (non-QPU-specific).
    pub fn evaluate(&self, action: &SovereignAction) -> Decision {
        // 1. Mental privacy guard: never allow raw neural shard reads to cross LLM boundary
        if matches!(action.kind, SovereignActionKind::ReadNeuralShard)
            && self.neurorights.mental_privacy
        {
            return Decision::Deny {
                reason: "Mental privacy: direct neural shard export is forbidden".into(),
            };
        }

        // 2. Forbid key read by any AI-mediated route
        if matches!(action.kind, SovereignActionKind::ReadKeys) {
            return Decision::Deny {
                reason: "Keys may not be read via AI-mediated routes".into(),
            };
        }

        // 3. Lifeforce / bioscale constraint example
        if action.lifeforce_cost > self.max_lifeforce_for_route(&action.route) {
            return Decision::Deny {
                reason: format!(
                    "Lifeforce envelope exceeded for route {}",
                    action.route
                ),
            };
        }

        // 4. Example of constrained allowance: redact fields that violate neurorights
        let mut redactions = Vec::new();
        if self.neurorights.dreamstate_sensitive
            && action.requested_fields.iter().any(|f| f.contains("dream"))
        {
            redactions.push("dream_segments".to_string());
        }

        if redactions.is_empty() {
            Decision::Allow {
                reason: "Allowed within neurorights and lifeforce envelopes".into(),
            }
        } else {
            Decision::AllowWithConstraints {
                reason: "Redacted dreamstate-sensitive fields".into(),
                redactions,
            }
        }
    }

    fn max_lifeforce_for_route(&self, route: &str) -> f32 {
        match route {
            "BCI" => 0.2,
            "OTA" => 0.3,
            "GOV" => 0.5,
            _ => 0.1,
        }
    }

    /// QPU / neuromorph-specific evaluation using Quantum Sovereignty Envelope.
    pub fn evaluate_quantum(
        &self,
        snapshot: &QuantumRuntimeSnapshot,
        req: &QuantumWorkloadRequest,
    ) -> Decision {
        let qenv = QuantumSovereigntyEnvelope::new(&self.tsafe);
        qenv.evaluate(snapshot, req)
    }
}
