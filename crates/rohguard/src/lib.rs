pub struct RoHModel {
    pub ceiling: f32,
    pub weights: std::collections::HashMap<String, f32>,
}

pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub struct RoHGuard {
    model: RoHModel,
}

impl RoHGuard {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let model: RoHModel = serde_json::from_str(&text)?;
        anyhow::ensure!((model.ceiling - 0.3).abs() < 1e-6, "RoH ceiling must be 0.3");
        Ok(Self { model })
    }

    pub fn check(&self, action: XRAction) -> Result<(), GuardError> {
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
