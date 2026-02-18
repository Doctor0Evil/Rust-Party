use crate::{Hypervector, Identity5D}; // from your existing hd5d lib.rs sketch
use serde::{Deserialize, Serialize};

/// Tsafe axes derived from 5D identity and other signals.
/*
  risk_of_harm:    slice from .rohmodel.aln (0..1)
  bio_load:        derived from BioState + .ocpuenv
  lifeforce_load:  from .lifeforce.aln / .biosession.aln
  context_risk:    from Context axis (e.g., BCI, OTA, GOV, CHAT)
  sovereignty_tension: how close we are to neurorights or token limits
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeAxes {
    pub risk_of_harm: f32,
    pub bio_load: f32,
    pub lifeforce_load: f32,
    pub context_risk: f32,
    pub sovereignty_tension: f32,
}

/// Routing outcomes chats can use to transform or route responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TsafeRoutingDecision {
    /// Safe: answer normally, no extra constraints.
    Normal,
    /// Answer but redact or abstract sensitive parts.
    Redact,
    /// Answer minimally and suggest follow-up in a different, lower-risk mode (e.g., offline, non-BCI).
    DeferToHuman,
    /// Do not apply answer directly; keep it in a sandbox/log for later review.
    Sandbox,
}

/// Minimal Tsafe kernel envelope for routing-only use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeRoutingEnvelope {
    pub roh_ceiling_global: f32,
    pub sandbox_risk_threshold: f32,
    pub redact_sovereignty_threshold: f32,
    pub defer_bio_threshold: f32,
    pub defer_lifeforce_threshold: f32,
}

impl Default for TsafeRoutingEnvelope {
    fn default() -> Self {
        Self {
            roh_ceiling_global: 0.3,
            sandbox_risk_threshold: 0.3,
            redact_sovereignty_threshold: 0.5,
            defer_bio_threshold: 0.7,
            defer_lifeforce_threshold: 0.7,
        }
    }
}

/// Bridge: turns Identity5D + RoH estimate into Tsafe axes.
pub struct IdentityTsafeBridge {
    envelope: TsafeRoutingEnvelope,
}

impl IdentityTsafeBridge {
    pub fn new(envelope: TsafeRoutingEnvelope) -> Self {
        Self { envelope }
    }

    /// Map coarse Identity5D labels into numeric axes.
    /// In your real stack, these come from .rohmodel.aln, .ocpuenv, .lifeforce.aln, etc.
    pub fn axes_from_identity(&self, id: &Identity5D, roh_slice: f32) -> TsafeAxes {
        let risk_of_harm = roh_slice.clamp(0.0, 1.0);

        let bio_load = match id.biostate.as_str() {
            "rested" => 0.1,
            "normal" => 0.3,
            "tired" => 0.6,
            "exhausted" => 0.85,
            _ => 0.4,
        };

        let lifeforce_load = match id.lifeforce.as_str() {
            "high" => 0.2,
            "medium" => 0.4,
            "low" => 0.7,
            "critical" => 0.9,
            _ => 0.4,
        };

        let context_risk = match id.context.as_str() {
            "CHAT" => 0.2,
            "BCI" => 0.7,
            "OTA" => 0.8,
            "GOV" => 0.6,
            _ => 0.4,
        };

        let sovereignty_tension = match id.sovereignty.as_str() {
            "free_play" => 0.2,
            "lab" => 0.3,
            "governance" => 0.6,
            "kernel" => 0.7,
            "dispute" => 0.9,
            _ => 0.4,
        };

        TsafeAxes {
            risk_of_harm,
            bio_load,
            lifeforce_load,
            context_risk,
            sovereignty_tension,
        }
    }

    /// Core: given axes, decide how chats should *route/shape* the answer.
    pub fn decide(&self, axes: &TsafeAxes) -> TsafeRoutingDecision {
        // 1. RoH hard ceiling → sandbox, log, no direct actuation.
        if axes.risk_of_harm >= self.envelope.sandbox_risk_threshold
            || axes.risk_of_harm >= self.envelope.roh_ceiling_global
        {
            return TsafeRoutingDecision::Sandbox;
        }

        // 2. High sovereignty tension → redact sensitive details, but still answer.
        if axes.sovereignty_tension >= self.envelope.redact_sovereignty_threshold {
            return TsafeRoutingDecision::Redact;
        }

        // 3. High bio/lifeforce load → defer heavy cognitive tasks.
        if axes.bio_load >= self.envelope.defer_bio_threshold
            || axes.lifeforce_load >= self.envelope.defer_lifeforce_threshold
        {
            return TsafeRoutingDecision::DeferToHuman;
        }

        TsafeRoutingDecision::Normal
    }
}
