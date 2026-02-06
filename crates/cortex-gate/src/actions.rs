use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignAction {
    ReadNeuralShard { shard_id: String },
    ProposeEvolveKernel { model_id: String, proposal_hash: String },
    ApplyOtaUpdate { package_url: String },
    SignTransaction { tx_bytes: Vec<u8> },
    DraftNeurorightsPolicy { draft: String },
    LogHighRiskEvent { details: String },
}
