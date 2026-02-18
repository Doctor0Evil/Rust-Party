use crate::spec::{NeuralProtection, NeurorightsFlags};
use serde::{Deserialize, Serialize};

/// Simplified IO operation kind to guard.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsOp {
    Open,
    Read,
    Write,
    Export,
    Delete,
}

/// Minimal descriptor for a file request that guards can inspect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsAccessContext {
    pub path: String,
    pub op: FsOp,
    pub is_ai_route: bool,
    pub neurorights: NeurorightsFlags,
    /// Approximate bio-load cost (0..1) for this operation.
    pub lifeforce_cost: f32,
}

/// Result of a guard decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GuardDecision {
    Allow,
    Deny { reason: String },
}

/// Trait for a single named protection.
pub trait FsGuard: Send + Sync {
    fn name(&self) -> NeuralProtection;
    fn check(&self, ctx: &FsAccessContext) -> GuardDecision;
}

/// AuraBoundaryGuard – prevents cross-subject or remote access to neural shards.
pub struct AuraBoundaryGuard;

impl FsGuard for AuraBoundaryGuard {
    fn name(&self) -> NeuralProtection {
        NeuralProtection::AuraBoundaryGuard
    }

    fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        if ctx.is_ai_route && ctx.neurorights.mentalprivacy && matches!(ctx.op, FsOp::Read | FsOp::Export) {
            return GuardDecision::Deny {
                reason: "AuraBoundaryGuard: mentalprivacy forbids AI-mediated neural shard reads/exports".into(),
            };
        }
        GuardDecision::Allow
    }
}

/// SoulNonTradeableShield – blocks export/tokenization of non-tradeable shards.
pub struct SoulNonTradeableShield;

impl FsGuard for SoulNonTradeableShield {
    fn name(&self) -> NeuralProtection {
        NeuralProtection::SoulNonTradeableShield
    }

    fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        if ctx.neurorights.soulnontradeable && matches!(ctx.op, FsOp::Export) {
            return GuardDecision::Deny {
                reason: "SoulNonTradeableShield: soulnontradeable forbids export/tokenization".into(),
            };
        }
        GuardDecision::Allow
    }
}

/// DreamSanctumFilter – prevents dreamstate data from affecting decisions.
pub struct DreamSanctumFilter;

impl FsGuard for DreamSanctumFilter {
    fn name(&self) -> NeuralProtection {
        NeuralProtection::DreamSanctumFilter
    }

    fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        if ctx.neurorights.dreamstatesensitive
            && ctx.neurorights.forbiddecisionuse
            && ctx.is_ai_route
            && matches!(ctx.op, FsOp::Read | FsOp::Export)
        {
            return GuardDecision::Deny {
                reason: "DreamSanctumFilter: dreamstate-sensitive data cannot be used for decision-making routes".into(),
            };
        }
        GuardDecision::Allow
    }
}

/// BioLoadThrottle – denies or throttles IO when lifeforce cost is too high.
pub struct BioLoadThrottle {
    pub max_cost: f32,
}

impl FsGuard for BioLoadThrottle {
    fn name(&self) -> NeuralProtection {
        NeuralProtection::BioLoadThrottle
    }

    fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        if ctx.lifeforce_cost > self.max_cost {
            return GuardDecision::Deny {
                reason: format!(
                    "BioLoadThrottle: lifeforce cost {} exceeds max {}",
                    ctx.lifeforce_cost, self.max_cost
                ),
            };
        }
        GuardDecision::Allow
    }
}

/// SovereignKernelLock – protects SOVEREIGNCONFIG shards from direct mutation.
pub struct SovereignKernelLock;

impl FsGuard for SovereignKernelLock {
    fn name(&self) -> NeuralProtection {
        NeuralProtection::SovereignKernelLock
    }

    fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        let is_root_or_config = ctx
            .path
            .contains("rohmodel.aln")
            || ctx.path.contains("stake.aln")
            || ctx.path.contains("neurorights.json")
            || ctx.path.contains("smart.json")
            || ctx.path.contains("evolve-token.json")
            || ctx.path.contains("tsafe.aln")
            || ctx.path.contains("vkernel.aln");
        if is_root_or_config && matches!(ctx.op, FsOp::Write | FsOp::Delete) {
            return GuardDecision::Deny {
                reason: "SovereignKernelLock: SOVEREIGNCONFIG shards may only change via EVOLVE+donutloop, not direct writes".into(),
            };
        }
        GuardDecision::Allow
    }
}

/// Composite guard that runs multiple named protections in sequence.
pub struct CompositeGuard {
    guards: Vec<Box<dyn FsGuard>>,
}

impl CompositeGuard {
    pub fn new(guards: Vec<Box<dyn FsGuard>>) -> Self {
        Self { guards }
    }

    pub fn check(&self, ctx: &FsAccessContext) -> GuardDecision {
        for g in &self.guards {
            match g.check(ctx) {
                GuardDecision::Allow => continue,
                deny @ GuardDecision::Deny { .. } => return deny,
            }
        }
        GuardDecision::Allow
    }
}
