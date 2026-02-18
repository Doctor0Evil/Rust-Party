use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod alnschemas;
pub mod guardians;
pub mod firewall;

use alnschemas::{NeurorightsPolicy, RohModel};
use firewall::{FirewallDecision, MetaFirewall};
use guardians::{EcoGuard, NeurorightsGuard, RohGuard};

/// Shared error type used by all guards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

/// XR / sovereign action kind enumeration.
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

/// Typed action used by Tsafe for XR / BCI / OTA flows.
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

/// Capability kinds for AI / tool calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityKind {
    SummarizeText,
    DraftEvolveProposal,
    ExplainPolicy,
    XRRoutePlan,
}

/// Short‑lived, typed capability descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub id: Uuid,
    pub kind: CapabilityKind,
    pub subject_id: String,
    pub max_tokens: u32,
    pub expires_at_unix: i64,
    /// "SuggestOnly", "ConfigOnly", etc.
    pub actuation_rights: String,
}

impl CapabilityChord {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now > self.expires_at_unix
    }
}

/// High‑level request into Tsafe Cortex Gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Bostrom / zeta / 0x519f...
    pub subject_id: String,
    /// "BCI", "OTA", "GOV", "CHAT", "XR"
    pub route: String,
    /// Only for LLM‑mediated flows.
    pub raw_prompt: Option<String>,
    /// Typed, non‑text action.
    pub action: XRAction,
    pub capability: CapabilityChord,
}

/// Authorized action with optional constraints (redactions, rate limits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedAction {
    pub action: XRAction,
    pub constraints: Vec<String>,
}

/// Structured rejection reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionReason {
    pub code: String,
    pub message: String,
}

/// Final authorization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationResult {
    Authorized(AuthorizedAction),
    Rejected(RejectionReason),
}

/// Minimal placeholder: later, append‑only writer to shards/ledger/donutloop.aln.
pub struct DonutloopLogger {
    log_path: std::path::PathBuf,
}

impl DonutloopLogger {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self { log_path: path.as_ref().to_path_buf() }
    }

    pub fn log_allow(&self, _req: &Request) {
        // TODO: append hash‑linked entry to .donutloop.aln
    }

    pub fn log_reject(&self, _req: &Request, _code: &str) {
        // TODO: append rejection entry to .donutloop.aln
    }

    pub fn log_blocked(&self, _req: &Request, _reason: &str) {
        // TODO: append firewall‑block entry
    }
}

/// Main Tsafe Cortex Gate type for XR / AI‑chat integration.
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
        // 1. Tsafe / NeuralTrust‑style firewall over text.
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
            return AuthorizationResult::Rejected(RejectionReason {
                code: err.code,
                message: err.message,
            });
        }

        // 4. RoH guard (0.3 ceiling, monotone safety).
        if let Err(err) = self.roh_guard.check(&req.action) {
            self.donutlogger.log_reject(&req, &err.code);
            return AuthorizationResult::Rejected(RejectionReason {
                code: err.code,
                message: err.message,
            });
        }

        // 5. Eco / lifeforce envelopes via .ocpuenv, .lifeforce.aln.
        if let Err(err) = self.eco_guard.check(&req.action, &req.route) {
            self.donutlogger.log_reject(&req, &err.code);
            return AuthorizationResult::Rejected(RejectionReason {
                code: err.code,
                message: err.message,
            });
        }

        // TODO: 6. EVOLVE token verification for structural / OTA actions.

        self.donutlogger.log_allow(&req);
        AuthorizationResult::Authorized(AuthorizedAction {
            action: req.action,
            constraints: Vec::new(),
        })
    }
}
