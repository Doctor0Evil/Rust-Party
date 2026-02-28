use crate::capabilities::SovereignClassification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPosture {
    pub mental_privacy: bool,
    pub dream_state_sensitive: bool,
    pub forbid_decision_use: bool,
    pub soul_non_tradeable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionProposalRecord {
    pub schema_version: u32,
    pub proposal_id: String,
    pub prev_hexstamp: Option<String>,
    pub hexstamp: String,
    pub timestamp: String,
    pub subject_id: String,
    pub description: String,
    pub roh_before: f32,
    pub roh_after: f32,
    pub neurorights_posture: NeurorightsPosture,
    pub token_kind: String,
    pub bostrom_tx: String,
    pub consent_proofs: Vec<String>,
    pub affected_shards: Vec<String>,
    pub sovereign_classification: SovereignClassification,
}

impl EvolutionProposalRecord {
    /// CI helper: basic invariants the record must satisfy.
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.roh_after > self.roh_before {
            return Err("RoH monotone safety violated (roh_after > roh_before)".into());
        }
        if self.roh_after > 0.3 {
            return Err("RoH ceiling 0.3 violated".into());
        }
        if self.sovereign_classification.required_token_kind.to_string() != self.token_kind {
            return Err("Token kind does not match sovereign classification".into());
        }
        Ok(())
    }
}

// Convenience Display for TokenKind so the string compare above works.
use crate::capabilities::TokenKind;
impl ToString for TokenKind {
    fn to_string(&self) -> String {
        match self {
            TokenKind::Smart => "Smart".into(),
            TokenKind::Evolve => "Evolve".into(),
            TokenKind::Chat => "Chat".into(),
            TokenKind::Other(s) => s.clone(),
        }
    }
}
