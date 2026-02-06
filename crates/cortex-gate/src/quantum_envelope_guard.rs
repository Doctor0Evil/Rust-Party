use serde::{Deserialize, Serialize};

use crate::policy::{Decision};
use crate::tsafe::TsafeKernel;

/// Telemetry snapshot from your QPU / neuromorph runtime shards.
/// These values should be populated by reading OrganicCpuQpuRuntime*.aln
/// and related bioscale logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumRuntimeSnapshot {
    pub qpu_roh: f32,
    pub qpu_coherence: f32,
    pub qpu_eco_impact: f32,
    pub lifeforce_load: f32,
    pub roh_global: f32,
}

/// Metadata about a proposed QPU/neuromorph operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumWorkloadRequest {
    /// Logical route: e.g. "BCI", "OTA", "CHAT", "GOV".
    pub route: String,
    /// Human-readable description or ID.
    pub label: String,
    /// Estimated incremental contributions from this workload.
    pub delta_qpu_roh: f32,
    pub delta_coherence: f32,
    pub delta_eco_impact: f32,
    pub delta_lifeforce: f32,
}

/// Named guard: Quantum Sovereignty Envelope.
pub struct QuantumSovereigntyEnvelope<'a> {
    tsafe: &'a TsafeKernel,
}

impl<'a> QuantumSovereigntyEnvelope<'a> {
    pub fn new(tsafe: &'a TsafeKernel) -> Self {
        Self { tsafe }
    }

    /// Evaluate whether a proposed quantum/neuromorph workload is allowed
    /// under the current snapshot and Tsafe / vkernel envelopes.
    pub fn evaluate(
        &self,
        snapshot: &QuantumRuntimeSnapshot,
        req: &QuantumWorkloadRequest,
    ) -> Decision {
        // Pull configured bounds from Tsafe.
        let (qpu_roh_min, qpu_roh_max) =
            self.tsafe.get_axis_bounds("qpu_roh").unwrap_or((0.0, 0.20));
        let (qpu_coh_min, qpu_coh_max) =
            self.tsafe.get_axis_bounds("qpu_coherence").unwrap_or((0.0, 0.60));
        let (_eco_min, eco_max) =
            self.tsafe.get_axis_bounds("qpu_eco_impact").unwrap_or((0.0, 0.50));
        let (_lf_min, lf_max) =
            self.tsafe.get_axis_bounds("lifeforce_load").unwrap_or((0.0, 0.80));
        let (_roh_min, roh_max) =
            self.tsafe.get_axis_bounds("roh_global").unwrap_or((0.0, 0.30));

        // Compute projected state after this workload.
        let proj_qpu_roh = snapshot.qpu_roh + req.delta_qpu_roh;
        let proj_coh = snapshot.qpu_coherence + req.delta_coherence;
        let proj_eco = snapshot.qpu_eco_impact + req.delta_eco_impact;
        let proj_lf = snapshot.lifeforce_load + req.delta_lifeforce;
        let proj_roh = snapshot.roh_global + req.delta_qpu_roh;

        // Hard caps: global RoH and quantum-specific RoH must not be exceeded.
        if proj_qpu_roh > qpu_roh_max || proj_roh > roh_max {
            return Decision::Deny {
                reason: format!(
                    "Quantum Sovereignty Envelope: RoH ceiling exceeded (proj_qpu_roh={:.3}, proj_roh={:.3})",
                    proj_qpu_roh, proj_roh
                ),
            };
        }

        // Coherence and lifeforce coupling (matches vkernel constraint).
        if proj_lf + 0.5 * proj_coh > 1.0 || proj_lf > lf_max || proj_coh > qpu_coh_max {
            return Decision::Deny {
                reason: format!(
                    "Quantum Sovereignty Envelope: lifeforce/coherence envelope exceeded \
                     (proj_lf={:.3}, proj_coh={:.3})",
                    proj_lf, proj_coh
                ),
            };
        }

        // Eco impact bound.
        if proj_eco > eco_max {
            return Decision::Deny {
                reason: format!(
                    "Quantum Sovereignty Envelope: eco-impact envelope exceeded (proj_eco={:.3})",
                    proj_eco
                ),
            };
        }

        // Optional: apply tighter limits on certain routes.
        if req.route == "BCI" && proj_qpu_roh > qpu_roh_min + 0.5 * (qpu_roh_max - qpu_roh_min) {
            return Decision::AllowWithConstraints {
                reason: format!(
                    "Quantum Sovereignty Envelope: BCI route near RoH ceiling (proj_qpu_roh={:.3}); \
                     downgrade workload.",
                    proj_qpu_roh
                ),
                redactions: vec!["high_intensity_protocols".to_string()],
            };
        }

        Decision::Allow {
            reason: "Quantum Sovereignty Envelope: workload within envelopes".into(),
        }
    }
}
