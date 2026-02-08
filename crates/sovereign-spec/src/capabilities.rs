#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BiophysicalScope {
    None,
    ReadOnly,          // biophysical-read-only
    EnvelopeOnly,      // Envelope-only evolution
    FullActuationForbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActuationRights {
    SuggestOnly,       // Suggest-only / non-actuating
    ConfigOnly,
    NoMotorControl,    // default in your shell
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SafetyProfile {
    MonotoneSafetyUpdate,        // G_new ≤ G_old, D_new ≤ D_old
    ExperimentalWithinEnvelopes, // still RoH- and envelope-bound
    RequiresHostMultisig,        // EVOLVE + Host/OrganicCPU + civic role
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RightsProfile {
    NeurorightsBound,   // neurorights.json + neurorights ALN
    NonCommercialNeuro, // non-commercial-neuro
    PublicAdvisoryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SovereignClassification {
    pub biophysical_scope: BiophysicalScope,
    pub actuation_rights: ActuationRights,
    pub safety_profile: SafetyProfile,
    pub rights_profile: RightsProfile,
}
