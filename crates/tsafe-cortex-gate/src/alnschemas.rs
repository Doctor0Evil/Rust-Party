use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPolicy {
    pub mentalprivacy: bool,
    pub cognitiveliberty: bool,
    pub forbiddecisionuse: bool,
    pub dreamstatesensitive: bool,
    pub soulnontradeable: bool,
    pub storagescope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RohModel {
    pub ceiling: f32,                       // must be 0.3
    pub weights: HashMap<String, f32>,
}

impl RohModel {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let model: RohModel = serde_json::from_str(&text)?;
        anyhow::ensure!((model.ceiling - 0.3).abs() < 1e-6, "RoH ceiling must be 0.3");
        Ok(model)
    }
}

pub fn load_neurorights<P: AsRef<Path>>(path: P) -> anyhow::Result<NeurorightsPolicy> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}
