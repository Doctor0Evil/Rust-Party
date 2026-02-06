use std::{fs::OpenOptions, io::Write, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FsOpKind {
    Open,
    Copy,
    Export,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsOpContext {
    pub subject_id: String, // Bostrom address
    pub route: String,      // "BCI", "OTA", "CHAT", "GOV", "LOCAL"
    pub kind: FsOpKind,
    pub path: PathBuf,
    pub dest: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonutloopEntry {
    pub proposal_id: String,
    pub decision: String,
    pub roh_before: f32,
    pub roh_after: f32,
    pub effect_bounds: String,
    pub token_kind: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum ShieldError {
    #[error("operation denied: {0}")]
    Denied(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct ShieldConfig {
    pub sovereign_roots: Vec<PathBuf>,
    pub protected_exts: Vec<String>, // [".neuroaln", ".lifeforce.aln", ...]
    pub donutloop_path: PathBuf,
}

#[derive(Debug)]
pub struct SoulNonTradeableShield {
    cfg: ShieldConfig,
}

impl SoulNonTradeableShield {
    pub fn new(cfg: ShieldConfig) -> Self {
        Self { cfg }
    }

    fn is_protected(&self, path: &Path) -> bool {
        let ext_match = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .map(|ext| self.cfg.protected_exts.iter().any(|p| p == &ext))
            .unwrap_or(false);

        let root_match = self
            .cfg
            .sovereign_roots
            .iter()
            .any(|root| path.starts_with(root));

        ext_match || root_match
    }

    fn log_denial(&self, ctx: &FsOpContext, reason: &str) -> Result<(), ShieldError> {
        let entry = DonutloopEntry {
            proposal_id: format!("fs-deny-{}", ctx.kind as u8),
            decision: "denied".into(),
            roh_before: 0.1,
            roh_after: 0.1,
            effect_bounds: "fs-export".into(),
            token_kind: "FsGuard".into(),
            timestamp: Utc::now(),
            reason: reason.into(),
        };
        let line = serde_json::to_string(&entry).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cfg.donutloop_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn check(&self, ctx: &FsOpContext) -> Result<(), ShieldError> {
        if !self.is_protected(&ctx.path) {
            return Ok(());
        }

        // Local, non-export reads by the host may still be allowed.
        let is_export_like = matches!(ctx.kind, FsOpKind::Copy | FsOpKind::Export);
        let is_ai_route = ctx.route != "LOCAL";

        if is_export_like || is_ai_route {
            let reason = format!(
                "SoulNonTradeableShield blocked {:?} on {:?} via route {}",
                ctx.kind, ctx.path, ctx.route
            );
            self.log_denial(ctx, &reason)?;
            return Err(ShieldError::Denied(reason));
        }

        Ok(())
    }
}

// Neuro-eXpFS middleware hook
pub trait GuardedVfs {
    fn guarded_open(&self, ctx: &FsOpContext) -> Result<(), ShieldError>;
}

pub struct NeuroExpFsWithShield {
    shield: SoulNonTradeableShield,
}

impl NeuroExpFsWithShield {
    pub fn new(shield: SoulNonTradeableShield) -> Self {
        Self { shield }
    }
}

impl GuardedVfs for NeuroExpFsWithShield {
    fn guarded_open(&self, ctx: &FsOpContext) -> Result<(), ShieldError> {
        self.shield.check(ctx)
    }
}
