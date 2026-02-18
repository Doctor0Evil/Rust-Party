use serde::{Deserialize, Serialize};
use std::time::Duration;

/// High-level block class – how blocks on disk are grouped.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsBlockClass {
    Generic,
    NeuroStream,
    BioSpec,
    Ledger,
    Model,
    SovereignConfig,
}

/// File type class, roughly mirroring eXpFS plus neuromorph additions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsFileType {
    Root,
    Data,
    Exec,
    NeuroStream,
    BioSpec,
    Ledger,
    Model,
    SovereignConfig,
}

/// Neurorights flags attached to each shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsFlags {
    pub mentalprivacy: bool,
    pub mentalintegrity: bool,
    pub cognitiveliberty: bool,
    pub noncommercialneuraldata: bool,
    pub soulnontradeable: bool,
    pub dreamstatesensitive: bool,
    pub forbiddecisionuse: bool,
    /// Hours after which deletion is allowed (forget SLA).
    pub forgetslahours: u32,
}

/// SMART tuning scope – small, reversible day-to-day adjustments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartScope {
    pub maxeffectsizel2: f32,
    pub domains: Vec<String>,
    pub expiry: Option<Duration>,
    pub physioguard_enabled: bool,
    pub revocable: bool,
}

/// EVOLVE requirement – governs deep structural evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolveRequirement {
    pub required: bool,
    pub scope_paths: Vec<String>,
    pub roh_ceiling: f32,
}

/// Governance descriptors for a shard class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardGovernance {
    pub neurorights: NeurorightsFlags,
    pub smart_scope: Option<SmartScope>,
    pub evolve: EvolveRequirement,
}

/// Named neural protections – enforced at FS and Tsafe gates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NeuralProtection {
    AuraBoundaryGuard,
    SoulNonTradeableShield,
    DreamSanctumFilter,
    BioLoadThrottle,
    SovereignKernelLock,
}

/// Spec for one shard class (e.g., NEUROSTREAM, MODEL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardClassSpec {
    pub file_type: FsFileType,
    pub block_class: FsBlockClass,
    pub extensions: Vec<String>,
    pub description: String,
    pub governance: ShardGovernance,
    pub protections: Vec<NeuralProtection>,
}

/// Full FS spec – neuromorph view on top of a classic disk layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicCpuFsSpec {
    pub disk_block_words: u32,
    pub shard_classes: Vec<ShardClassSpec>,
}

impl OrganicCpuFsSpec {
    /// Convenience helper to find the spec for a given file extension.
    pub fn class_for_extension(&self, ext: &str) -> Option<&ShardClassSpec> {
        self.shard_classes
            .iter()
            .find(|c| c.extensions.iter().any(|e| e == ext))
    }
}
