use crate::protections::{JurisdictionCapsule, NeuralProtection, RouteActuationClass};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfranetTokenScope {
    pub has_smart: bool,
    pub has_evolve: bool,
    pub has_chat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoHSlice {
    pub value: f32,      // 0.0..=0.3
    pub monotone_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfranetMeta {
    pub subject_bostrom: String,       // subject address, not IP
    pub host_ocpu_id: String,          // OrganicCPU host identifier
    pub actuation_class: RouteActuationClass,
    pub token_scope: InfranetTokenScope,
    pub protections: Vec<NeuralProtection>,
    pub roh_slice: RoHSlice,
    pub jurisdiction: JurisdictionCapsule,
    pub neuro_ip: bool,                // NeuroIP-bearing payload
}
