use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioTelem {
    pub knowledge_factor_k: f32,       // 0.0–1.0
    pub host_energy_d: f32,            // normalized demand
    pub psych_risk_dw: f32,            // 0.0–1.0
    pub roh_estimate: f32,             // current RoH slice
    pub lifeforce_index: f32,          // 0.0–1.0
    pub thermal_distance_index: f32,   // 0.0–1.0
    pub molecular_balance_index: f32,  // 0.0–1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BioLoadFlag {
    Normal,
    Caution,
    Violation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmMode {
    Normal,
    Caution,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanosotinEnvelope {
    pub roh_ceiling: f32,          // must be <= 0.3
    pub max_d: f32,                // energy demand bound
    pub max_dw: f32,               // psych-risk bound
    pub min_lifeforce: f32,        // minimum lifeforce
    pub max_thermal_distance: f32, // keep below overheating
    pub min_molecular_balance: f32,// biochemical stability floor
}
