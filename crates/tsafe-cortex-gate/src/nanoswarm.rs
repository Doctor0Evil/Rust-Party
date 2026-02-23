use serde::{Deserialize, Serialize};
use crate::policy::{PolicyEngine, Decision};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRGridStep {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub host_budget_cost: f32,   // HostBudget per step
    pub lifeforce_delta: f32,    // estimated effect on LifeforceIndex
    pub roh_delta: f32,          // estimated RoH change
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoswarmAction {
    pub subject_id: String,        // Bostrom / DID
    pub route: String,             // "XR-NANOSWARM"
    pub xr_step: XRGridStep,
    pub biotelem: crate::bioscale::BioTelemSnapshot,
}

impl PolicyEngine {
    pub fn evaluate_nanoswarm_action(&self, act: &NanoswarmAction) -> Decision {
        // Enforce HostBudget & LifeforceIndex at control-plane level.
        if act.xr_step.host_budget_cost > self.budget.host_budget_remaining {
            return Decision::Deny {
                reason: "HostBudget exceeded for XR step".into(),
            };
        }
        if act.biotelem.lifeforce_index + act.xr_step.lifeforce_delta
            < self.budget.min_lifeforce_index
        {
            return Decision::Deny {
                reason: "Lifeforce envelope would be violated".into(),
            };
        }

        // RoH monotone & ceiling.
        let projected_roh = act.biotelem.roh_estimate + act.xr_step.roh_delta;
        if !self.roh_model.is_within_ceiling(projected_roh) {
            return Decision::Deny {
                reason: "RoH ceiling 0.3 would be exceeded".into(),
            };
        }
        if !self.roh_model.is_monotone(act.biotelem.roh_estimate, projected_roh) {
            return Decision::Deny {
                reason: "RoH monotone safety violated".into(),
            };
        }

        // Governance shards are *not* re-evaluated here; they were compiled
        // into self.budget and self.route_caps at startup.

        Decision::Allow
    }
}
