use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    SummarizeShard,
    DraftEvolveProposal,
    ExplainPolicy,
    SuggestSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub kind: CapabilityKind,
    pub subject_id: String,
    pub max_tokens: u32,
    pub expires_at_unix: i64,
    pub actuation_rights: String, // always "SuggestOnly"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub capability: CapabilityChord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
}

pub trait LlmClient {
    fn speak_only(&self, req: &LlmRequest) -> anyhow::Result<LlmResponse>;
}
