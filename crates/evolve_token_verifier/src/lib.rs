use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolveToken {
    pub token_id: String,
    pub subject_id: String,
    pub scopes: Vec<String>,
    pub roh_before: f32,
    pub roh_after: f32,
    pub signatures: Vec<String>,
    pub hexstamp: String,
}

#[derive(Debug, Clone)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub trait BostromClient {
    fn verify_evolve_token(&self, token_id: &str, subject_id: &str) -> anyhow::Result<bool>;
}

pub struct EvolveGuard<C: BostromClient> {
    client: C,
}

impl<C: BostromClient> EvolveGuard<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn check(&self, action: &crate::XRAction, subject_id: &str) -> Result<(), GuardError> {
        if matches!(action.kind, crate::XRActionKind::ProposeEvolve | crate::XRActionKind::ApplyOta) {
            // In a full impl, the action carries token_id; simplified here.
            let token_id = "from_action_context";
            let ok = self
                .client
                .verify_evolve_token(token_id, subject_id)
                .unwrap_or(false);
            if !ok {
                return Err(GuardError {
                    code: "EVOLVE_TOKEN_INVALID".into(),
                    message: "Missing or invalid EVOLVE token for structural change".into(),
                });
            }
        }
        Ok(())
    }
}
