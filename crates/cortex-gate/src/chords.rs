use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewMode {
    Redacted,
    FullAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadShardChord {
    pub target_shard_id: String,
    pub view_mode: ViewMode,
    pub context_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeEvolveChord {
    pub model_id: String,
    pub source_code_hash: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub id: Uuid,
    pub subject_id: String, // Bostrom address
    pub route: String,      // BCI, OTA, GOV, CHAT
    pub max_tokens: u32,
    pub expires_at_unix: i64,
}
