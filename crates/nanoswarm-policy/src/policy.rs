use crate::state::{BioTelem, BioLoadFlag, SwarmMode, NanosotinEnvelope};

pub struct NanoswarmPolicyEngine {
    env: NanosotinEnvelope,
}

impl NanoswarmPolicyEngine {
    pub fn new(env: NanosotinEnvelope) -> Self {
        assert!(env.roh_ceiling <= 0.3 + 1e-6);
        Self { env }
    }

    pub fn classify_bioload(&self, t: &BioTelem) -> BioLoadFlag {
        // Hard violation checks – any one triggers Violation.
        if t.roh_estimate > self.env.roh_ceiling
            || t.host_energy_d > self.env.max_d
            || t.psych_risk_dw > self.env.max_dw
            || t.lifeforce_index < self.env.min_lifeforce
            || t.thermal_distance_index > self.env.max_thermal_distance
            || t.molecular_balance_index < self.env.min_molecular_balance
        {
            return BioLoadFlag::Violation;
        }

        // Caution band: within envelopes but close to edges.
        let roh_margin = self.env.roh_ceiling - t.roh_estimate;
        let d_margin = self.env.max_d - t.host_energy_d;
        let dw_margin = self.env.max_dw - t.psych_risk_dw;
        let lf_margin = t.lifeforce_index - self.env.min_lifeforce;

        if roh_margin < 0.05 || d_margin < 0.05 || dw_margin < 0.05 || lf_margin < 0.05 {
            BioLoadFlag::Caution
        } else {
            BioLoadFlag::Normal
        }
    }

    pub fn decide_mode(&self, t: &BioTelem) -> SwarmMode {
        match self.classify_bioload(t) {
            BioLoadFlag::Normal => SwarmMode::Normal,
            BioLoadFlag::Caution => SwarmMode::Caution,
            BioLoadFlag::Violation => SwarmMode::Rollback,
        }
    }
}
