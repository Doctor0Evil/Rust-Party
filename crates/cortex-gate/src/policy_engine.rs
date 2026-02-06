use crate::actions::SovereignAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BostromIdentity {
    pub subject_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("insufficient EVOLVE stake")]
    InsufficientStake,
    #[error("not an approved signer")]
    NotAuthorized,
    #[error("violates neurorights or RoH limits")]
    NeurorightsViolation,
}

pub trait PolicyEngine {
    fn can_execute(
        &self,
        action: &SovereignAction,
        identity: &BostromIdentity,
    ) -> Result<(), PolicyError>;
}
