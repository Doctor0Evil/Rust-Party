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
pub struct TsafeAxis {
    pub name: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeKernel {
    pub axes: Vec<TsafeAxis>,    // binds .tsafe.aln / .vkernel.aln
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RohModel {
    pub ceiling: f32,
    pub weights: HashMap<String, f32>,
}

pub fn load_neurorights<P: AsRef<Path>>(path: P) -> anyhow::Result<NeurorightsPolicy> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn load_rohmodel<P: AsRef<Path>>(path: P) -> anyhow::Result<RohModel> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn load_tsafe_kernel<P: AsRef<Path>>(path: P) -> anyhow::Result<TsafeKernel> {
    // For now assume JSON-compatible representation of .tsafe.aln.
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
