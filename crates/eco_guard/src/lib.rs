use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoEnvelope {
    pub max_power_watts: f32,
    pub max_heat_joules: f32,
}

#[derive(Debug, Clone)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub struct EcoGuard {
    envelopes_by_route: std::collections::HashMap<String, EcoEnvelope>,
}

impl EcoGuard {
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let envs: std::collections::HashMap<String, EcoEnvelope> = serde_json::from_str(&text)?;
        Ok(Self { envelopes_by_route: envs })
    }

    pub fn check(&self, action: &crate::XRAction, route: &str) -> Result<(), GuardError> {
        if let Some(env) = self.envelopes_by_route.get(route) {
            if action.lifeforce_cost > env.max_power_watts {
                return Err(GuardError {
                    code: "ECO_ENVELOPE_EXCEEDED".into(),
                    message: "Eco / lifeforce envelope exceeded for route".into(),
                });
            }
        }
        Ok(())
    }
}
