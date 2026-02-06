//! Sovereign-neuroaura: NeuroAura Boundary Lattice (NABL) enforcement.
//! Guards spatial/temporal envelopes for neural I/O and neuromorphic stimulation.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Spatial envelope in meters relative to subject center.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialEnvelope {
    pub radius_m: f32,
}

/// Temporal envelope for duty cycle and session length.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalEnvelope {
    pub max_session_duration_ms: u64,
    pub max_duty_cycle_percent: f32,
}

/// Carrier and modulation constraints for BCI links.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CarrierEnvelope {
    pub min_hz: f32,
    pub max_hz: f32,
    pub allowed_modulations: Vec<String>,
}

/// Full NABL shard as stored in .neuroaura-boundary.aln.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeuroAuraBoundary {
    pub subject_id: String,
    pub spatial: SpatialEnvelope,
    pub temporal: TemporalEnvelope,
    pub carrier: CarrierEnvelope,
}

#[derive(Debug, Clone)]
pub struct StimulationRequest {
    pub subject_id: String,
    pub duration: Duration,
    pub duty_cycle_percent: f32,
    pub carrier_hz: f32,
    pub modulation: String,
}

#[derive(Debug, Clone)]
pub struct NeuroAuraDecision {
    pub allowed: bool,
    pub reason: String,
}

impl NeuroAuraBoundary {
    pub fn evaluate(&self, req: &StimulationRequest) -> NeuroAuraDecision {
        if self.subject_id != req.subject_id {
            return NeuroAuraDecision {
                allowed: false,
                reason: "subject_mismatch".into(),
            };
        }

        if req.duration.as_millis() as u64 > self.temporal.max_session_duration_ms {
            return NeuroAuraDecision {
                allowed: false,
                reason: "session_duration_exceeds_boundary".into(),
            };
        }

        if req.duty_cycle_percent > self.temporal.max_duty_cycle_percent {
            return NeuroAuraDecision {
                allowed: false,
                reason: "duty_cycle_exceeds_boundary".into(),
            };
        }

        if req.carrier_hz < self.carrier.min_hz || req.carrier_hz > self.carrier.max_hz {
            return NeuroAuraDecision {
                allowed: false,
                reason: "carrier_out_of_bounds".into(),
            };
        }

        if !self
            .carrier
            .allowed_modulations
            .iter()
            .any(|m| m == &req.modulation)
        {
            return NeuroAuraDecision {
                allowed: false,
                reason: "modulation_not_allowed".into(),
            };
        }

        NeuroAuraDecision {
            allowed: true,
            reason: "ok".into(),
        }
    }
}
