//! Core sovereign types: neurorights, RoH, OrganicCPU envelopes and policies.
//! This crate is pure types + basic logic; no I/O or platform bindings.

use serde::{Deserialize, Serialize};

/// High-level error domain for sovereignty guards.
#[derive(Debug, thiserror::Error)]
pub enum SovereignError {
    #[error("neurorights policy violation")]
    NeurorightsViolation,
    #[error("risk-of-harm model invalid or exceeded")]
    RoHModelInvalid,
    #[error("bioscale / OrganicCPU envelope violation")]
    BioEnvelopeViolation,
    #[error("resource limit exceeded")]
    ResourceLimitExceeded,
    #[error("risk too high for sensitive domain")]
    RiskTooHigh,
    #[error("consent too weak for neurorights domain")]
    ConsentTooWeak,
    #[error("workspace resolution failed: {0}")]
    WorkspaceError(String),
}

/// Risk-of-Harm scalar with soft bounds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct RoH(pub f32);

impl RoH {
    pub fn capped(self, ceiling: f32) -> Self {
        RoH(self.0.min(ceiling))
    }

    pub fn zero() -> Self {
        RoH(0.0)
    }
}

/// RoH model shard (simplified surface).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoHModel {
    /// Global ceiling, typically 0.3 in your design.
    pub global_ceiling: RoH,
}

impl RoHModel {
    pub fn global_ceiling(&self) -> f32 {
        self.global_ceiling.0
    }
}

/// Neurorights policy derived from .neurorights.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeurorightsPolicy {
    pub mental_privacy: bool,
    pub mental_integrity: bool,
    pub cognitive_liberty: bool,
    pub noncommercial_neural_data: bool,
    pub soulnontradeable: bool,
    pub dreamstate_sensitive: bool,
    pub forbid_decision_use: bool,
    /// Minimum number of explanation tokens for valid consent text.
    pub min_explanation_tokens: u32,
    /// Whether current domain is neurorights-sensitive (BCI, OTA, medical, etc.).
    pub domain_sensitive: bool,
    /// Whether telemetry can be disabled entirely.
    pub allow_disable_telemetry: bool,
}

impl NeurorightsPolicy {
    pub fn permits_telemetry(&self, disable_telemetry: bool) -> bool {
        if disable_telemetry && !self.allow_disable_telemetry {
            return false;
        }
        true
    }

    pub fn domain_is_sensitive(&self) -> bool {
        self.domain_sensitive
    }

    pub fn min_explanation_tokens(&self) -> u32 {
        self.min_explanation_tokens
    }
}

/// OrganicCPU environment envelope from .ocpuenv, .lifeforce.aln, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcpuEnv {
    pub max_threads: u32,
    pub max_memory_bytes: u64,
    pub max_duty_cycle_percent: f32,
}

impl OcpuEnv {
    pub fn max_threads_for_duty_cycle(&self, current_duty_cycle: f32) -> f32 {
        if current_duty_cycle >= self.max_duty_cycle_percent {
            0.0
        } else {
            self.max_threads as f32
        }
    }

    pub fn max_memory_bytes(&self) -> u64 {
        self.max_memory_bytes
    }
}

/// Runtime metrics shard (OrganicCpuRuntimeMetrics… .aln).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMetrics {
    pub current_duty_cycle: f32,
    pub fatigue_index: f32,
    pub eco_impact: f32,
}

/// Aggregated envelopes for convenience.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelopes {
    pub ocpuenv: OcpuEnv,
    pub runtime_metrics: RuntimeMetrics,
}

/// Workspace manifest identity for lookup: neuro-workspace.manifest.aln, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRef {
    pub subject_id: String,
    pub workspace_id: String,
}
