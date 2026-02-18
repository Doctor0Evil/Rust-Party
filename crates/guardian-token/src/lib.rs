use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use sovereign_kernel::types::{GuardError, GuardResult, SovereignAction, SovereignActionKind, TokenVerifier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolveToken {
    pub scope_paths: Vec<String>,
    pub roh_before: f32,
    pub roh_after: f32,
    pub token_kind: String,      // "QPolicyUpdate" | "BioScaleUpgrade" | ...
    pub signatures: Vec<String>, // Bostrom DIDs
    pub hexstamp: String,
}

pub trait BostromVerifier {
    fn verify_on_chain(&self, token: &EvolveToken) -> anyhow::Result<bool>;
}

pub struct TokenVerifierImpl<V: BostromVerifier> {
    evolve_token: EvolveToken,
    verifier: V,
}

impl<V: BostromVerifier> TokenVerifierImpl<V> {
    pub fn from_file(path: &Path, verifier: V) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;
        let token: EvolveToken = serde_json::from_str(&text)?;
        Ok(Self {
            evolve_token: token,
            verifier,
        })
    }
}

impl<V: BostromVerifier> TokenVerifier for TokenVerifierImpl<V> {
    fn verify_evolve_token(&self, action: &SovereignAction) -> GuardResult {
        if !matches!(
            action.kind,
            SovereignActionKind::ProposeEvolve | SovereignActionKind::ApplyOta
        ) {
            return Ok(());
        }

        // Structural / OTA changes must carry valid EVOLVE token
        let ok = self
            .verifier
            .verify_on_chain(&self.evolve_token)
            .map_err(|e| GuardError {
                code: "EVOLVE_VERIFY_ERR".into(),
                message: e.to_string(),
            })?;
        if !ok {
            return Err(GuardError {
                code: "EVOLVE_INVALID".into(),
                message: "EVOLVE token invalid or not confirmed on Bostrom chain.".into(),
            });
        }

        Ok(())
    }
}
