use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NeuralProtection {
    // Filesystem-origin guards, reused at network layer
    AuraBoundaryGuard,
    SoulNonTradeableShield,
    DreamSanctumFilter,
    BioLoadThrottle,
    SovereignKernelLock,

    // Infranet-series mesh protections
    InfranetSovereignMeshGuard,
    RoHMeshCeiling,
    NeuroIPTransitShield,
    SovereignJurisdictionCapsule,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouteActuationClass {
    BiophysicalReadOnly,  // metrics, summaries, indexes only
    EnvelopeOnly,         // adjust envelopes/limits, no direct actuation
    NonActuating,         // pure planning / sim, no physical side-effects
    Actuating,            // requires EVOLVE + Tsafe + donutloop approval
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionCapsule {
    pub region_id: String,        // e.g. "phoenix-az-us"
    pub legal_profile_id: String, // refs ALN shard describing local law
    pub lab_only: bool,
}
