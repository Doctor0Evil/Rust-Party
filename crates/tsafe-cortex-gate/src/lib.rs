use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod alnschemas;
pub mod guardians;
pub mod firewall;

use alnschemas::{NeurorightsPolicy, RohModel};
use firewall::{FirewallDecision, MetaFirewall};
use guardians::{EcoGuard, NeurorightsGuard, RohGuard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XRActionKind {
    ReadNeuralShard,
    WriteNeuralShard,
    ProposeEvolve,
    ApplyOta,
    XRRouteStep,
    ScheduleJob,
    ReadKeys,
    SignTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRAction {
    pub kind: XRActionKind,
    pub subject_id: String,
    pub route: String,          // "BCI", "OTA", "GOV", "CHAT", "XR"
    pub requested_fields: Vec<String>,
    pub lifeforce_cost: f32,
    pub roh_before: f32,
    pub roh_after_estimate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    SummarizeText,
    DraftEvolveProposal,
    ExplainPolicy,
    XRRoutePlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub id: Uuid,
    pub kind: CapabilityKind,
    pub subject_id: String,
    pub max_tokens: u32,
    pub expires_at_unix: i64,
    pub actuation_rights: String, // "SuggestOnly", "ConfigOnly"
}

impl CapabilityChord {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now > self.expires_at_unix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub subject_id: String,          // Bostrom / zeta / 0x519f...
    pub route: String,               // "BCI", "OTA", "GOV", "CHAT", "XR"
    pub raw_prompt: Option<String>,  // only for LLM-mediated flows
    pub action: XRAction,            // typed, non-text action
    pub capability: CapabilityChord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedAction {
    pub action: XRAction,
    pub constraints: Vec<String>, // e.g. redactions, rate limits
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionReason {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationResult {
    Authorized(AuthorizedAction),
    Rejected(RejectionReason),
}

pub struct DonutloopLogger {
    // minimal placeholder: later, append-only writer to shards/ledger/donutloop.aln
    log_path: std::path::PathBuf,
}

impl DonutloopLogger {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self { log_path: path.as_ref().to_path_buf() }
    }

    pub fn log_allow(&self, _req: &Request) {
        // TODO: append hash-linked entry to .donutloop.aln
    }

    pub fn log_reject(&self, _req: &Request, _code: &str) {
        // TODO: append rejection entry to .donutloop.aln
    }

    pub fn log_blocked(&self, _req: &Request, _reason: &str) {
        // TODO: append firewall-block entry
    }
}

pub struct TsafeCortexGate {
    firewall: MetaFirewall,
    roh_guard: RohGuard,
    nr_guard: NeurorightsGuard,
    eco_guard: EcoGuard,
    donutlogger: DonutloopLogger,
}

impl TsafeCortexGate {
    pub fn new(
        firewall: MetaFirewall,
        roh_guard: RohGuard,
        nr_guard: NeurorightsGuard,
        eco_guard: EcoGuard,
        donutlogger: DonutloopLogger,
    ) -> Self {
        Self { firewall, roh_guard, nr_guard, eco_guard, donutlogger }
    }

    pub fn authorize(&self, req: Request) -> AuthorizationResult {
        // 1. Tsafe / NeuralTrust-style firewall over text.
        if let Some(prompt) = &req.raw_prompt {
            match self.firewall.evaluate_prompt(prompt, &req.route) {
                FirewallDecision::Block => {
                    self.donutlogger.log_blocked(&req, "firewall_block");
                    return AuthorizationResult::Rejected(RejectionReason {
                        code: "FIREWALL_BLOCK".into(),
                        message: "Prompt blocked by Tsafe Cortex Gate firewall".into(),
                    });
                }
                FirewallDecision::Quarantine => {
                    self.donutlogger.log_blocked(&req, "firewall_quarantine");
                    return AuthorizationResult::Rejected(RejectionReason {
                        code: "FIREWALL_QUARANTINE".into(),
                        message: "Prompt requires human review".into(),
                    });
                }
                FirewallDecision::Allow => {}
            }
        }

        // 2. CapabilityChord gate (actuation is never granted to CHAT / SuggestOnly).
        if req.capability.is_expired() {
            self.donutlogger.log_reject(&req, "CAPABILITY_EXPIRED");
            return AuthorizationResult::Rejected(RejectionReason {
                code: "CAPABILITY_EXPIRED".into(),
                message: "Capability chord is expired".into(),
            });
        }
        if req.capability.actuation_rights == "SuggestOnly"
            && matches!(req.action.kind, XRActionKind::ApplyOta | XRActionKind::ProposeEvolve)
        {
            self.donutlogger.log_reject(&req, "CAPABILITY_SUGGEST_ONLY");
            return AuthorizationResult::Rejected(RejectionReason {
                code: "CAPABILITY_SUGGEST_ONLY".into(),
                message: "This capability cannot actuate OTA or EVOLVE".into(),
            });
        }

        // 3. Neurorights guard (mental privacy, dreamstate, soulnontradeable).
        if let Err(err) = self.nr_guard.check_action(&req.action, &req.route) {
            self.donutlogger.log_reject(&req, &err.code);
            return AuthorizationResult::Rejected(err);
        }

        // 4. RoH guard (0.3 ceiling, monotone safety).
        if let Err(err) = self.roh_guard.check(&req.action) {
            self.donutlogger.log_reject(&req, &err.code);
            return AuthorizationResult::Rejected(err);
        }

        // 5. Eco / lifeforce envelopes via .ocpuenv, .lifeforce.aln.
        if let Err(err) = self.eco_guard.check(&req.action, &req.route) {
            self.donutlogger.log_reject(&req, &err.code);
            return AuthorizationResult::Rejected(err);
        }

        // TODO: 6. EVOLVE token verification for structural / OTA actions.

        self.donutlogger.log_allow(&req);
        AuthorizationResult::Authorized(AuthorizedAction {
            action: req.action,
            constraints: Vec::new(),
        })
    }
}
