use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// High-level boundary descriptor tying Bostrom identity, NeuroXFS shard classes,
/// and ALN policy shards into a single Sovereign Execution Boundary surface.
///
/// This crate is intentionally "spec-heavy": it is the shared vocabulary
/// that Tsafe Cortex Gate, NeuroXFS, and simulators use to agree on where
/// non-transferable agency starts and ends.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BostromIdentity {
    /// Canonical subject root (e.g., bostrom18sd2u…).
    pub primary_address: String,
    /// Optional secure / monitored alternate (e.g., bostrom1ldgmtf…).
    pub secure_address: Option<String>,
    /// Additional safe alternates (zeta…, 0x… anchors).
    pub safe_alternates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardClass {
    NeuroStream,     // .neuroaln, .nstream.neuroaln, .lifaln
    BioSpec,         // .ocpu, .ocpuenv, .lifeforce.aln, .biosession.aln
    Model,           // .nnetx, .nnetw, .nnetq, .nfeat.aln, .nmap.aln
    Ledger,          // .donutloop.aln, .evolve.jsonl, .nnet-loop.aln, .answer.ndjson
    SovereignConfig, // .rohmodel.aln, .tsafe.aln, .vkernel.aln, .stake.aln, .neurorights.json, .smart.json, .evolve-token.json
    Proof,           // .bchainproof.json, .nnet-proof.bchain.json
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardBinding {
    /// Logical class (governs which guards and ALN schemas apply).
    pub class: ShardClass,
    /// On-disk path relative to the subject's NeuroXFS root.
    pub path: PathBuf,
    /// Whether raw bytes may ever cross an AI / LLM boundary.
    pub ai_export_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnPolicyShards {
    pub rohmodel_path: PathBuf,       // .rohmodel.aln (RoH ≤ 0.3 ceiling + axes)
    pub tsafe_path: PathBuf,          // .tsafe.aln (Tsafe kernels / viability regions)
    pub vkernel_path: PathBuf,        // .vkernel.aln
    pub neurorights_path: PathBuf,    // .neurorights.json
    pub smart_policy_path: PathBuf,   // .smart.json
    pub evolve_token_path: PathBuf,   // .evolve-token.json
    pub stake_path: PathBuf,          // .stake.aln
    pub manifest_path: PathBuf,       // neuro-workspace.manifest.aln
}

/// Invariants that the Sovereign Execution Boundary must enforce for this node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryInvariants {
    /// Global RoH ceiling; must be 0.3 for sovereign Rust-Party nodes.
    pub roh_ceiling: f32,
    /// Whether RoH monotonicity (no increase without EVOLVE) is required.
    pub enforce_roh_monotonicity: bool,
    /// Whether neurorights flags are hard blockers (not advisory).
    pub enforce_neurorights: bool,
    /// Whether Donutloop must be append-only and hash-linked.
    pub enforce_donutloop_append_only: bool,
}

/// Full, typed description of a Rust-Party Sovereign Execution Boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignExecutionBoundary {
    pub subject: BostromIdentity,
    pub shard_bindings: Vec<ShardBinding>,
    pub aln_policies: AlnPolicyShards,
    pub invariants: BoundaryInvariants,
}

impl SovereignExecutionBoundary {
    /// Minimal health check that can be asserted in CI and at node boot.
    pub fn validate_static_invariants(&self) -> Result<(), String> {
        if (self.invariants.roh_ceiling - 0.3).abs() > 1e-6 {
            return Err("RoH ceiling must be fixed at 0.3 for sovereign nodes".into());
        }
        if !self.invariants.enforce_donutloop_append_only {
            return Err("Donutloop append-only invariant must be enabled".into());
        }
        // Example: ensure SovereignConfig shards are never AI-exportable.
        for shard in &self.shard_bindings {
            if matches!(shard.class, ShardClass::SovereignConfig) && shard.ai_export_allowed {
                return Err(format!(
                    "SOVEREIGNCONFIG shard {:?} must not be AI-exportable",
                    shard.path
                ));
            }
        }
        Ok(())
    }
}
