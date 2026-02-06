use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use aura_boundary_guard::CapabilityKind; // reuse enum for global cohesion

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioMetrics {
    pub roh_level: f32,      // 0.0 - 1.0
    pub fatigue_score: f32,  // 0.0 - 1.0
    pub lifeforce_index: f32 // 0.0 - 1.0
}

#[derive(Debug, Error)]
pub enum BioError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct BioConfig {
    pub ocpuenv_path: String,
    pub biosession_path: String,
    pub lifeforce_path: String,
    pub roh_ceiling: f32,
    pub fatigue_ceiling: f32,
}

#[derive(Debug)]
pub struct BioLoadThrottle {
    cfg: BioConfig,
}

impl BioLoadThrottle {
    pub fn new(cfg: BioConfig) -> Self {
        Self { cfg }
    }

    fn load_metrics(&self) -> Result<BioMetrics, BioError> {
        // For Rust-Party, keep this simple: one JSON per path, merged.
        fn read_json<P: AsRef<Path>, T: for<'de> Deserialize<'de>>(p: P) -> Result<T, BioError> {
            let s = std::fs::read_to_string(p)?;
            Ok(serde_json::from_str(&s)?)
        }

        let env: BioMetrics = read_json(&self.cfg.ocpuenv_path)?;
        let session: BioMetrics = read_json(&self.cfg.biosession_path)?;
        let life: BioMetrics = read_json(&self.cfg.lifeforce_path)?;

        Ok(BioMetrics {
            roh_level: env.roh_level.max(session.roh_level).max(life.roh_level),
            fatigue_score: env.fatigue_score
                .max(session.fatigue_score)
                .max(life.fatigue_score),
            lifeforce_index: env.lifeforce_index
                .min(session.lifeforce_index)
                .min(life.lifeforce_index),
        })
    }

    pub fn get_available_capabilities(
        &self,
        all: &[CapabilityKind],
    ) -> Result<Vec<CapabilityKind>, BioError> {
        let m = self.load_metrics()?;

        let mut caps = Vec::new();
        for k in all {
            let allowed = match k {
                CapabilityKind::DraftEvolveProposal => {
                    m.roh_level < self.cfg.roh_ceiling * 0.8
                        && m.fatigue_score < self.cfg.fatigue_ceiling * 0.7
                }
                CapabilityKind::SignTransaction | CapabilityKind::ApplyOtaUpdate => {
                    m.roh_level < self.cfg.roh_ceiling * 0.6
                        && m.fatigue_score < self.cfg.fatigue_ceiling * 0.5
                }
                _ => true,
            };

            if allowed {
                caps.push(k.clone());
            }
        }
        Ok(caps)
    }
}
