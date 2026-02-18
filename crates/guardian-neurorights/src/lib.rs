use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use sovereign_kernel::types::{GuardError, GuardResult, NeurorightsGuard, SovereignActionKind, SovereignAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPolicy {
    pub mentalprivacy: bool,
    pub mentalintegrity: bool,
    pub cognitiveliberty: bool,
    pub noncommercialneuraldata: bool,
    pub soulnontradeable: bool,
    pub dreamstatesensitive: bool,
    pub forbiddecisionuse: bool,
    pub storagescope: String,
}

pub struct NeurorightsGuardImpl {
    policy: NeurorightsPolicy,
}

impl NeurorightsGuardImpl {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let policy: NeurorightsPolicy = serde_json::from_str(&text)?;
        Ok(Self { policy })
    }
}

impl NeurorightsGuard for NeurorightsGuardImpl {
    fn check_rights(&self, action: &SovereignAction) -> GuardResult {
        // Example 1: mental privacy vs raw neural export on AI routes.
        if self.policy.mentalprivacy
            && matches!(action.kind, SovereignActionKind::ReadNeuralShard)
            && matches!(action.route.as_str(), "CHAT" | "BCI")
        {
            return Err(GuardError {
                code: "NR_MENTAL_PRIVACY".into(),
                message: "Direct neural shard export via AI route is forbidden.".into(),
            });
        }

        // Example 2: forbid decision use of dream-derived features on governance/OTA.
        let touches_dream = action
            .requested_fields
            .iter()
            .any(|f| f.contains("dream"));
        if self.policy.dreamstatesensitive
            && self.policy.forbiddecisionuse
            && touches_dream
            && matches!(action.route.as_str(), "GOV" | "OTA")
        {
            return Err(GuardError {
                code: "NR_DREAM_FORBID".into(),
                message: "Dream-derived data cannot be used for decisions.".into(),
            });
        }

        Ok(())
    }
}
