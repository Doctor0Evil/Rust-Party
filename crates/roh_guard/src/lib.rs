use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoHModel {
    pub ceiling: f32,                      // must be 0.3
    pub weights: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub struct RoHGuard {
    model: RoHModel,
}

impl RoHGuard {
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let model: RoHModel = serde_json::from_str(&text)?;
        anyhow::ensure!((model.ceiling - 0.3).abs() < 1e-6, "RoH ceiling must be 0.3");
        Ok(Self { model })
    }

    pub fn calculate_and_check_roh_impact(
        &self,
        action: &crate::XRAction,
    ) -> Result<(), GuardError> {
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
                message: "RoH monotone safety violated; RoH would increase".into(),
            });
        }
        Ok(())
    }
}
