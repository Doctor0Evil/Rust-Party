use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignActionKind {
    ReadNeuralShard,
    WriteNeuralShard,
    ProposeEvolve,
    ApplyOta,
    ReadKeys,
    SignTransaction,
    InfranetPacket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAction {
    pub kind: SovereignActionKind,
    pub subject_id: String,       // Bostrom address
    pub route: String,            // "BCI" | "OTA" | "GOV" | "CHAT" | "INFRA"
    pub requested_fields: Vec<String>,
    pub lifeforce_cost: f32,
    pub roh_before: f32,
    pub roh_after_estimate: f32,
}

#[derive(Debug, Clone)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

pub type GuardResult = Result<(), GuardError>;

pub trait RoHGuard {
    fn check_harm(&self, action: &SovereignAction) -> GuardResult;
}

pub trait NeurorightsGuard {
    fn check_rights(&self, action: &SovereignAction) -> GuardResult;
}

pub trait EcoGuard {
    fn check_load(&self, action: &SovereignAction) -> GuardResult;
}

pub trait TokenVerifier {
    fn verify_evolve_token(
        &self,
        action: &SovereignAction,
    ) -> GuardResult;
}

pub trait InfranetGuard {
    fn validate_packet(&self, action: &SovereignAction) -> GuardResult;
}
