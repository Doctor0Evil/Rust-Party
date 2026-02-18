use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use sovereign_kernel::types::{GuardError, GuardResult, RoHGuard, SovereignAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RohModel {
    pub ceiling: f32,                       // should be 0.3
    pub weights: HashMap<String, f32>,      // optional axes
}

pub struct RoHGuardImpl {
    model: RohModel,
}

impl RoHGuardImpl {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let model: RohModel = serde_json::from_str(&text)?;
        anyhow::ensure!((model.ceiling - 0.3).abs() < 1e-6, "RoH ceiling must be 0.3");
        Ok(Self { model })
    }
}

impl RoHGuard for RoHGuardImpl {
    fn check_harm(&self, action: &SovereignAction) -> GuardResult {
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
                message: "RoH monotone safety violated (RoH would increase)".into(),
            });
        }
        Ok(())
    }
}
