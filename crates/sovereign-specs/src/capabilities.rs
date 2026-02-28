use serde::{Deserialize, Serialize};

/// High-level biophysical scope of an operation or proposal.
/// This is what CI keys on when deciding which envelopes and guards must apply.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BiophysicalScope {
    /// Purely informational / suggest-only; no direct biophysical effect.
    SuggestOnly,
    /// Small, reversible configuration or tuning (SMART scope).
    ReversibleTuning,
    /// Changes that affect lifeforce envelopes or pain / RoH axes.
    LifeforceEnvelope,
    /// Actions that can affect nanoswarm, XR, or OrganicCPU duty.
    ActuationPath,
    /// Structural changes to kernels, controllers, or biospec baselines.
    StructuralKernelChange,
    /// Reserved for future, more specific scopes (namespaced variants go here).
    #[serde(other)]
    Other,
}

/// What kind of actuation a capability or proposal is allowed to trigger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActuationRights {
    /// Text-only / non-actuating (CHAT-only).
    NonActuating,
    /// Can trigger local-only changes on the host, within envelopes.
    LocalHostOnly,
    /// Can trigger cross-node changes but never touch sovereign configs.
    FederatedNonSovereign,
    /// Can mutate sovereign configs (requires EVOLVE + multisig).
    SovereignConfigChange,
}

/// Safety profile class for an artifact or action.
/// CI and Tsafe Cortex Gate use this to decide which guards are mandatory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafetyProfile {
    /// Bounded RoH, within Tsafe kernels, with full guard coverage.
    BoundedRoHSafe,
    /// Experimental but still subject to RoH ≤ 0.3 and neurorights.
    ExperimentalBounded,
    /// Legacy / unknown; CI should block or quarantine by default.
    Unknown,
}

/// Rights profile ties into neurorights and data-use restrictions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RightsProfile {
    /// Mental privacy guarantees must be enforced.
    pub mental_privacy: bool,
    /// Dream-state data is sensitive and must not be used for decision-use.
    pub dream_state_sensitive: bool,
    /// Neural or identity-linked data may not be used for decisions
    /// in regulated domains (employment, housing, credit, etc.).
    pub forbid_decision_use: bool,
    /// Data is marked as soulnontradeable: no sale, no licensing, no export.
    pub soul_non_tradeable: bool,
    /// Additional namespaced flags (for jurisdictional or domain-specific rules).
    #[serde(default)]
    pub extra_flags: Vec<String>,
}

/// Token class required to enact a proposal or action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenKind {
    Smart,
    Evolve,
    Chat,
    /// For specialized token families (BIOSAFE, ECO, etc.).
    Other(String),
}

/// High-level classification used across proposals, actions, and models.
/// This is what CI should require on every evolution-capable artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SovereignClassification {
    /// Human-readable label, e.g. "SOVEREIGNCONFIG", "MODEL", "BIOSPEC", "LEDGER".
    pub class_label: String,
    /// Biophysical scope of the artifact or action.
    pub biophysical_scope: BiophysicalScope,
    /// Actuation rights associated with this artifact.
    pub actuation_rights: ActuationRights,
    /// Safety profile class.
    pub safety_profile: SafetyProfile,
    /// Rights profile, tied to neurorights and data use.
    pub rights_profile: RightsProfile,
    /// Token kind required to apply this artifact or action.
    pub required_token_kind: TokenKind,
}

impl SovereignClassification {
    /// Quick helper: CHAT-only, non-actuating classification.
    pub fn chat_non_actuating(label: &str, rights_profile: RightsProfile) -> Self {
        Self {
            class_label: label.to_string(),
            biophysical_scope: BiophysicalScope::SuggestOnly,
            actuation_rights: ActuationRights::NonActuating,
            safety_profile: SafetyProfile::BoundedRoHSafe,
            rights_profile,
            required_token_kind: TokenKind::Chat,
        }
    }

    /// Quick helper: SMART-tuning classification (no sovereign config).
    pub fn smart_tuning(label: &str, rights_profile: RightsProfile) -> Self {
        Self {
            class_label: label.to_string(),
            biophysical_scope: BiophysicalScope::ReversibleTuning,
            actuation_rights: ActuationRights::LocalHostOnly,
            safety_profile: SafetyProfile::BoundedRoHSafe,
            rights_profile,
            required_token_kind: TokenKind::Smart,
        }
    }

    /// Quick helper: EVOLVE-level structural change classification.
    pub fn evolve_structural(label: &str, rights_profile: RightsProfile) -> Self {
        Self {
            class_label: label.to_string(),
            biophysical_scope: BiophysicalScope::StructuralKernelChange,
            actuation_rights: ActuationRights::SovereignConfigChange,
            safety_profile: SafetyProfile::ExperimentalBounded,
            rights_profile,
            required_token_kind: TokenKind::Evolve,
        }
    }
}
