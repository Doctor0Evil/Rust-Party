use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    SummarizeText,
    DraftEvolveProposal,
    ExplainPolicy,
    XRRoutePlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub id: Uuid,
    pub kind: CapabilityKind,
    pub subject_id: String,
    pub max_tokens: u32,
    pub expires_at_unix: i64,
    pub actuation_rights: String, // "SuggestOnly", "ConfigOnly"
}

impl CapabilityChord {
    pub fn new(kind: CapabilityKind, subject_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            subject_id: subject_id.into(),
            max_tokens: 1024,
            expires_at_unix: chrono::Utc::now().timestamp() + 60,
            actuation_rights: "SuggestOnly".into(),
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at_unix
    }
}
