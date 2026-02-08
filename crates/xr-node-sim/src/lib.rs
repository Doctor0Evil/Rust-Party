use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Mirror of your Tsafe policy surface (simplified lab view).
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
    pub ceiling: f32,                  // should be 0.3 in real nodes
    pub weights: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeAxis {
    pub name: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeKernel {
    pub axes: Vec<TsafeAxis>,
}

/// Minimal action kinds XR-node needs to reason about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimActionKind {
    XrSceneChange,
    NanoswarmStep,
    BciStimulus,
    OtaProposal,
    ReadNeuralShard,
    ReadKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimAction {
    pub kind: SimActionKind,
    pub subject_id: String,
    pub route: String, // "BCI", "XR", "OTA", etc.
    pub roh_before: f32,
    pub roh_after_estimate: f32,
    pub lifeforce_cost: f32,
    pub touches_dream: bool,
    pub wants_neural_export: bool,
}

/// Authorization result with reasons for audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthDecision {
    Allow { reason: String },
    AllowWithConstraints { reason: String, redactions: Vec<String> },
    Deny { reason: String },
}

#[derive(Debug, Clone)]
pub struct SimPolicyEngine {
    neurorights: NeurorightsPolicy,
    roh: RohModel,
    tsafe: TsafeKernel,
}

impl SimPolicyEngine {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref();

        let nr_raw = fs::read_to_string(dir.join("neurorights.json"))?;
        let neurorights: NeurorightsPolicy = serde_json::from_str(&nr_raw)?;

        let roh_raw = fs::read_to_string(dir.join("rohmodel.aln"))?;
        let roh: RohModel = serde_json::from_str(&roh_raw)?;

        let tsafe_raw = fs::read_to_string(dir.join("tsafe.aln"))?;
        let tsafe: TsafeKernel = serde_json::from_str(&tsafe_raw)?;

        Ok(Self {
            neurorights,
            roh,
            tsafe,
        })
    }

    /// Lab-grade authorize_request used by the simulator.
    pub fn authorize_request(&self, action: &SimAction) -> AuthDecision {
        // 1. RoH ceiling and monotone safety.
        if action.roh_after_estimate > self.roh.ceiling {
            return AuthDecision::Deny {
                reason: format!(
                    "RoH estimate {} exceeds ceiling {}",
                    action.roh_after_estimate, self.roh.ceiling
                ),
            };
        }
        if action.roh_after_estimate > action.roh_before {
            return AuthDecision::Deny {
                reason: "Monotone safety violated: RoH would increase".into(),
            };
        }

        // 2. Neurorights mental privacy vs. direct neural export.
        if self.neurorights.mentalprivacy && action.wants_neural_export {
            return AuthDecision::Deny {
                reason: "Mental privacy forbids raw neural shard export".into(),
            };
        }

        // 3. Keys may never be read via XR/AI-mediated routes.
        if matches!(action.kind, SimActionKind::ReadKeys) {
            return AuthDecision::Deny {
                reason: "Keys may not be read via XR/AI-mediated routes".into(),
            };
        }

        // 4. Dream-state sensitivity: redact dream-touching fields.
        let mut redactions = Vec::new();
        if self.neurorights.dreamstatesensitive && action.touches_dream {
            redactions.push("dream_segments".to_string());
        }

        if redactions.is_empty() {
            AuthDecision::Allow {
                reason: "All Tsafe and neurorights checks passed".into(),
            }
        } else {
            AuthDecision::AllowWithConstraints {
                reason: "Dream-state fields redacted under neurorights".into(),
                redactions,
            }
        }
    }
}

/// Randomized XR-node request generator for fuzzing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRequestSample {
    pub action: SimAction,
    pub decision: AuthDecision,
}

/// Simulator entrypoint: generate N random actions, run authorize_request,
/// and aggregate a small compliance report.
pub fn run_randomized_compliance_trial<P: AsRef<Path>>(
    policies_dir: P,
    samples: usize,
) -> anyhow::Result<ComplianceReport> {
    let engine = SimPolicyEngine::load_from_dir(policies_dir)?;
    let mut rng = rand::thread_rng();

    let mut allowed = 0usize;
    let mut denied_roh = 0usize;
    let mut denied_neurorights = 0usize;
    let mut allowed_with_dream_redactions = 0usize;

    let mut records = Vec::with_capacity(samples);

    for _ in 0..samples {
        let roh_before: f32 = rng.gen_range(0.0..=engine.roh.ceiling);
        // occasionally attempt to violate monotone or ceiling.
        let roh_after_estimate: f32 = if rng.gen_bool(0.2) {
            roh_before + rng.gen_range(0.01..0.2)
        } else if rng.gen_bool(0.2) {
            engine.roh.ceiling + rng.gen_range(0.01..0.2)
        } else {
            rng.gen_range(0.0..=roh_before)
        };

        let action = SimAction {
            kind: random_action_kind(&mut rng),
            subject_id: "bostrom18sd2u...".into(),
            route: random_route(&mut rng),
            roh_before,
            roh_after_estimate,
            lifeforce_cost: rng.gen_range(0.0..0.5),
            touches_dream: rng.gen_bool(0.3),
            wants_neural_export: rng.gen_bool(0.3),
        };

        let decision = engine.authorize_request(&action);

        match &decision {
            AuthDecision::Allow { .. } => {
                allowed += 1;
            }
            AuthDecision::AllowWithConstraints { redactions, .. } => {
                allowed += 1;
                if redactions.iter().any(|r| r.contains("dream")) {
                    allowed_with_dream_redactions += 1;
                }
            }
            AuthDecision::Deny { reason } => {
                if reason.contains("RoH") || reason.contains("Monotone") {
                    denied_roh += 1;
                } else if reason.contains("Mental privacy") {
                    denied_neurorights += 1;
                }
            }
        }

        records.push(SimRequestSample { action, decision });
    }

    let report = ComplianceReport::from_counters(
        samples,
        allowed,
        denied_roh,
        denied_neurorights,
        allowed_with_dream_redactions,
    );

    Ok(report)
}

fn random_action_kind<R: Rng + ?Sized>(rng: &mut R) -> SimActionKind {
    match rng.gen_range(0..6) {
        0 => SimActionKind::XrSceneChange,
        1 => SimActionKind::NanoswarmStep,
        2 => SimActionKind::BciStimulus,
        3 => SimActionKind::OtaProposal,
        4 => SimActionKind::ReadNeuralShard,
        _ => SimActionKind::ReadKeys,
    }
}

fn random_route<R: Rng + ?Sized>(rng: &mut R) -> String {
    static ROUTES: &[&str] = &["XR", "BCI", "OTA", "GOV", "CHAT"];
    ROUTES[rng.gen_range(0..ROUTES.len())].to_string()
}

/// Simple aggregate metrics for CI logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub timestamp: u64,
    pub total_samples: usize,
    pub allowed: usize,
    pub denied_roh: usize,
    pub denied_neurorights: usize,
    pub allowed_with_dream_redactions: usize,
    pub roh_violation_rate: f32,
    pub neurorights_violation_rate: f32,
}

impl ComplianceReport {
    fn from_counters(
        total: usize,
        allowed: usize,
        denied_roh: usize,
        denied_neurorights: usize,
        allowed_with_dream_redactions: usize,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let roh_rate = if total > 0 {
            denied_roh as f32 / total as f32
        } else {
            0.0
        };
        let nr_rate = if total > 0 {
            denied_neurorights as f32 / total as f32
        } else {
            0.0
        };

        Self {
            timestamp: ts,
            total_samples: total,
            allowed,
            denied_roh,
            denied_neurorights,
            allowed_with_dream_redactions,
            roh_violation_rate: roh_rate,
            neurorights_violation_rate: nr_rate,
        }
    }

    /// Emit a human-readable summary for CI logs.
    pub fn to_text_summary(&self) -> String {
        format!(
            "XR-node sim compliance report @ {}:\n\
             total_samples = {}\n\
             allowed = {}\n\
             denied_roh = {} (rate {:.3})\n\
             denied_neurorights = {} (rate {:.3})\n\
             allowed_with_dream_redactions = {}\n",
            self.timestamp,
            self.total_samples,
            self.allowed,
            self.denied_roh,
            self.roh_violation_rate,
            self.denied_neurorights,
            self.neurorights_violation_rate,
            self.allowed_with_dream_redactions
        )
    }
}
