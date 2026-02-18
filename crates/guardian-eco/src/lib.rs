use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use sovereign_kernel::types::{GuardError, GuardResult, EcoGuard, SovereignAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoEnvelope {
    pub max_lifeforce_cost: f32,
}

pub struct EcoGuardImpl {
    envelopes_by_route: HashMap<String, EcoEnvelope>,
}

impl EcoGuardImpl {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let envs: HashMap<String, EcoEnvelope> = serde_json::from_str(&text)?;
        Ok(Self {
            envelopes_by_route: envs,
        })
    }
}

impl EcoGuard for EcoGuardImpl {
    fn check_load(&self, action: &SovereignAction) -> GuardResult {
        if let Some(env) = self.envelopes_by_route.get(&action.route) {
            if action.lifeforce_cost > env.max_lifeforce_cost {
                return Err(GuardError {
                    code: "ECO_ENVELOPE".into(),
                    message: "Lifeforce / eco envelope exceeded for route.".into(),
                });
            }
        }
        Ok(())
    }
}
