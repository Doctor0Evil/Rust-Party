use crate::capability::CapabilityChord;
use crate::firewall::{MetaFirewall, FirewallDecision};
use crate::guardians::GuardianSet;
use crate::donutloop::DonutloopLogger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub subject_id: String,        // Bostrom / zeta / 0x...
    pub route: String,             // "BCI", "OTA", "GOV", "CHAT", "XR"
    pub raw_prompt: Option<String>,// For LLM-mediated flows
    pub action: XRAction,          // Typed, non-text action
    pub capability: CapabilityChord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationResult {
    Authorized(AuthorizedAction),
    Rejected(RejectionReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedAction {
    pub action: XRAction,
    pub constraints: Vec<String>,  // e.g. redactions, rate limits
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionReason {
    pub code: String,
    pub message: String,
}

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
    pub route: String,
    pub requested_fields: Vec<String>,
    pub lifeforce_cost: f32,
    pub roh_before: f32,
    pub roh_after_estimate: f32,
}

pub struct TsafeCortexGate {
    firewall: MetaFirewall,
    guardians: GuardianSet,
    donutlogger: DonutloopLogger,
}

impl TsafeCortexGate {
    pub fn new(firewall: MetaFirewall, guardians: GuardianSet, donutlogger: DonutloopLogger) -> Self {
        Self { firewall, guardians, donutlogger }
    }

    pub fn authorize_request(&self, req: Request) -> AuthorizationResult {
        // 1. NeuralTrust-style firewall on text, if present.
        if let Some(prompt) = &req.raw_prompt {
            let decision = self.firewall.evaluate_prompt(prompt, &req.route);
            match decision {
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

        // 2. Capability chord parsing / route-level guard.
        if let Err(reason) = self.guardians.capability_guard.check_capability(&req.capability, &req.action, &req.route) {
            self.donutlogger.log_reject(&req, &reason.code);
            return AuthorizationResult::Rejected(reason);
        }

        // 3. Neurorights guard.
        if let Err(reason) = self.guardians.neurorights_guard.check(&req.action) {
            self.donutlogger.log_reject(&req, &reason.code);
            return AuthorizationResult::Rejected(reason);
        }

        // 4. RoH guard (monotonicity + ceiling 0.3).
        if let Err(reason) = self.guardians.roh_guard.check(&req.action) {
            self.donutlogger.log_reject(&req, &reason.code);
            return AuthorizationResult::Rejected(reason);
        }

        // 5. Eco guard (energy / heat envelopes).
        if let Err(reason) = self.guardians.eco_guard.check(&req.action, &req.route) {
            self.donutlogger.log_reject(&req, &reason.code);
            return AuthorizationResult::Rejected(reason);
        }

        // 6. EVOLVE token verifier for structural or OTA actions.
        if let Err(reason) = self.guardians.evolve_guard.check(&req.action, &req.subject_id) {
            self.donutlogger.log_reject(&req, &reason.code);
            return AuthorizationResult::Rejected(reason);
        }

        // 7. Success – log high-risk decisions into .donutloop.aln.
        self.donutlogger.log_allow(&req);

        AuthorizationResult::Authorized(AuthorizedAction {
            action: req.action.clone(),
            constraints: Vec::new(),
        })
    }
}
