use crate::{XRAction, XRActionKind};
use crate::alnschemas::{NeurorightsPolicy, RohModel};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

// ---- RoHGuard ----

pub struct RohGuard {
    model: RohModel,
}

impl RohGuard {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let model = RohModel::load(path)?;
        Ok(Self { model })
    }

    pub fn check(&self, action: &XRAction) -> Result<(), GuardError> {
        if action.roh_after_estimate > self.model.ceiling {
            return Err(GuardError {
                code: "ROH_CEILING".into(),
                message: format!(
                    "RoH estimate {} exceeds ceiling {}",
                    action.roh_after_estimate, self.model.ceiling
                ),
            });
        }
        if action.roh_after_estimate > action.roh_before {
            return Err(GuardError {
                code: "ROH_MONOTONE".into(),
                message: "RoH monotone safety violated (would increase)".into(),
            });
        }
        Ok(())
    }
}

// ---- NeurorightsGuard ----

pub struct NeurorightsGuard {
    policy: NeurorightsPolicy,
}

impl NeurorightsGuard {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let policy: NeurorightsPolicy = serde_json::from_str(&text)?;
        Ok(Self { policy })
    }

    pub fn check_action(&self, action: &XRAction, route: &str) -> Result<(), GuardError> {
        if self.policy.mentalprivacy
            && matches!(action.kind, XRActionKind::ReadNeuralShard)
            && (route == "CHAT" || route == "XR")
        {
            return Err(GuardError {
                code: "NEURORIGHTS_MENTAL_PRIVACY".into(),
                message: "Direct neural shard export via AI / XR route is forbidden".into(),
            });
        }
        if self.policy.dreamstatesensitive
            && route == "BCI"
            && matches!(action.kind, XRActionKind::ReadNeuralShard)
        {
            return Err(GuardError {
                code: "NEURORIGHTS_DREAM_FORBID".into(),
                message: "Dream-state sensitive neurorights forbid this access".into(),
            });
        }
        Ok(())
    }
}

// ---- EcoGuard (BioLoadThrottle) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoEnvelope {
    pub max_power_watts: f32,
    pub max_lifeforce_delta: f32,
}

pub struct EcoGuard {
    envelopes_by_route: HashMap<String, EcoEnvelope>,
}

impl EcoGuard {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let envs: HashMap<String, EcoEnvelope> = serde_json::from_str(&text)?;
        Ok(Self { envelopes_by_route: envs })
    }

    pub fn check(&self, action: &XRAction, route: &str) -> Result<(), GuardError> {
        if let Some(env) = self.envelopes_by_route.get(route) {
            if action.lifeforce_cost > env.max_lifeforce_delta {
                return Err(GuardError {
                    code: "ECO_ENVELOPE_EXCEEDED".into(),
                    message: "Lifeforce / eco envelope exceeded for route".into(),
                });
            }
        }
        Ok(())
    }
}
