use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroRightsPolicy {
    pub mental_privacy: bool,
    pub cognitive_liberty: bool,
    pub forbid_decision_use: bool,
    pub dreamstate_sensitive: bool,
    pub soul_non_tradeable: bool,
    pub storage_scope: String,
}

#[derive(Debug)]
pub struct NodeState {
    pub active_route: String,
    pub dreamstate_on: bool,
}

#[derive(Debug, Clone)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub struct NeurorightsGuard {
    policy: NeuroRightsPolicy,
}

impl NeurorightsGuard {
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let policy: NeuroRightsPolicy = serde_json::from_str(&text)?;
        Ok(Self { policy })
    }

    pub fn check_dream_state_access_allowed(&self, state: &NodeState) -> Result<(), GuardError> {
        if self.policy.dreamstate_sensitive && state.dreamstate_on {
            return Err(GuardError {
                code: "NEURORIGHTS_DREAM_FORBID".into(),
                message: "Dream-state sensitive flag forbids this access".into(),
            });
        }
        Ok(())
    }

    pub fn check_action(&self, action: &crate::XRAction) -> Result<(), GuardError> {
        // Example: no raw neural read on AI routes when mental_privacy is true.
        if self.policy.mental_privacy
            && matches!(action.kind, crate::XRActionKind::ReadNeuralShard)
            && action.route == "CHAT"
        {
            return Err(GuardError {
                code: "NEURORIGHTS_MENTAL_PRIVACY".into(),
                message: "Direct neural shard export via AI route is forbidden".into(),
            });
        }
        Ok(())
    }
}
