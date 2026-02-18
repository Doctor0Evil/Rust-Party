use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// --- Imports from your sovereign stack (existing or planned crates) ----

use tsafe_cortex_gate::{
    Request as TsafeRequest,
    XRAction,
    XRActionKind,
    AuthorizationResult,
    TsafeCortexGate,
};
use tsafe_cortex_gate::capability::{CapabilityChord, CapabilityKind};
use infranet_core::packet::{
    SovereignPacket,
    SovereignAddress,
    InfranetRouteKind,
    RoHSlice,
    NeurorightsEnvelope,
    TokenClass,
    CapabilityScope,
};
use infranet_core::packet::InfranetMeta;
use infranet_core::protections::{NeuralProtection, RouteActuationClass, JurisdictionCapsule};
use infranet_guard::{InfranetGuard, PolicyDecision};
use infranet_firewalls::{InfranetFirewall, FirewallVerdict};

// ---------- 1. Agent‑Verse identity and roles ---------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentVerseRole {
    ChatAgent,
    ResearchAgent,
    CivicFacilitator,
    SystemOrchestrator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentVerseIdentity {
    /// External id in Agent‑Verse / FetchHub.
    pub agent_id: String,
    /// Bostrom / zeta / 0x… mapped subject.
    pub subject_bostrom: String,
    /// Role drives CHAT / SMART / EVOLVE semantics.
    pub role: AgentVerseRole,
}

impl AgentVerseIdentity {
    pub fn tokencategory(&self) -> TokenClass {
        match self.role {
            AgentVerseRole::ChatAgent
            | AgentVerseRole::ResearchAgent
            | AgentVerseRole::CivicFacilitator => TokenClass::Chat,
            AgentVerseRole::SystemOrchestrator => TokenClass::Smart,
        }
    }

    /// CHAT roles are always non‑actuating in this adapter.
    pub fn route_actuation_class(&self) -> RouteActuationClass {
        match self.role {
            AgentVerseRole::ChatAgent
            | AgentVerseRole::ResearchAgent
            | AgentVerseRole::CivicFacilitator => RouteActuationClass::NonActuating,
            AgentVerseRole::SystemOrchestrator => RouteActuationClass::EnvelopeOnly,
        }
    }

    /// Map to a Tsafe capability kind for LLM‑style flows.
    pub fn default_capability_kind(&self) -> CapabilityKind {
        match self.role {
            AgentVerseRole::ChatAgent => CapabilityKind::SummarizeText,
            AgentVerseRole::ResearchAgent => CapabilityKind::ExplainPolicy,
            AgentVerseRole::CivicFacilitator => CapabilityKind::XRRoutePlan,
            AgentVerseRole::SystemOrchestrator => CapabilityKind::DraftEvolveProposal,
        }
    }
}

// ---------- 2. Agent‑Verse → sovereign request types --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentVerseTaskKind {
    /// Pure deliberation, no actuation.
    DraftPolicyText,
    /// Ask for explanation of an existing policy or shard.
    ExplainShard,
    /// Suggest OTA / evolution, never apply.
    SuggestEvolveProposal,
    /// Plan XR / nanoswarm route, non‑actuating.
    PlanXRRoute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentVerseTask {
    pub id: Uuid,
    pub created_at: SystemTime,
    pub identity: AgentVerseIdentity,
    pub kind: AgentVerseTaskKind,
    /// Natural‑language content from Agent‑Verse graph.
    pub prompt: String,
    /// Optional reference into local sovereign artifacts
    /// (e.g., "policies/rohmodel.aln", "state/evolve.jsonl#123").
    pub local_ref: Option<String>,
}

// High‑level result returned to Agent‑Verse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentVerseReply {
    pub task_id: Uuid,
    pub allowed: bool,
    pub quarantined: bool,
    pub reason: Option<String>,
    /// Textual answer or draft produced via Tsafe‑mediated LLM.
    pub text: Option<String>,
}

// ---------- 3. Adapter configuration ------------------------------------

#[derive(Clone)]
pub struct AgentVerseAdapterConfig {
    pub phoenix_region_capsule: JurisdictionCapsule,
    /// Default RoH slice used for CHAT‑only, non‑actuating flows.
    pub default_roh: RoHSlice,
}

impl Default for AgentVerseAdapterConfig {
    fn default() -> Self {
        Self {
            phoenix_region_capsule: JurisdictionCapsule {
                regionid: "phoenix-az-us".into(),
                legalprofileid: "phoenix-neurorights-v1".into(),
                labonly: false,
            },
            default_roh: RoHSlice {
                value: 0.05,
                monotoneok: true,
            },
        }
    }
}

// ---------- 4. Core adapter struct --------------------------------------

pub struct AgentVerseAdapter<G, F>
where
    G: AsRef<TsafeCortexGate> + Send + Sync,
    F: InfranetFirewallEngine + Send + Sync,
{
    tsafe_gate: G,
    infranet_guard: InfranetGuard,
    infranet_firewall: InfranetFirewall<F>,
    config: AgentVerseAdapterConfig,
}

// Simple trait alias to avoid pulling the full firewall crate into this file.
pub trait InfranetFirewallEngine: Send + Sync {
    fn classify(&self, text: &str) -> FirewallVerdict;
}

impl<G, F> AgentVerseAdapter<G, F>
where
    G: AsRef<TsafeCortexGate> + Send + Sync,
    F: InfranetFirewallEngine + Send + Sync,
{
    pub fn new(
        tsafe_gate: G,
        infranet_guard: InfranetGuard,
        infranet_firewall: InfranetFirewall<F>,
        config: AgentVerseAdapterConfig,
    ) -> Self {
        Self {
            tsafe_gate,
            infranet_guard,
            infranet_firewall,
            config,
        }
    }

    /// Main entry: accept an Agent‑Verse task, enforce Tsafe + Infranet
    /// invariants, and (optionally) call an LLM via Tsafe Cortex Gate.
    pub fn handle_task(&self, task: AgentVerseTask) -> AgentVerseReply {
        // 1. Build Infranet packet metadata for this CHAT‑style request.
        let pkt = self.build_packet(&task);

        // 2. Run Infranet firewall (NeuralTrust‑class) on the prompt label.
        let fw_verdict = self
            .infranet_firewall
            .evaluatepacket(&pkt);

        if matches!(fw_verdict, FirewallVerdict::Block) {
            return AgentVerseReply {
                task_id: task.id,
                allowed: false,
                quarantined: false,
                reason: Some("Agent‑Verse prompt blocked by Infranet firewall".into()),
                text: None,
            };
        }

        if matches!(fw_verdict, FirewallVerdict::Quarantine) {
            return AgentVerseReply {
                task_id: task.id,
                allowed: false,
                quarantined: true,
                reason: Some("Agent‑Verse prompt requires human review (quarantine)".into()),
                text: None,
            };
        }

        // 3. Enforce neurorights + RoH via InfranetGuard.
        let policy_decision = self.infranet_guard.evaluate(&pkt);
        match policy_decision {
            PolicyDecision::Deny { reason } => {
                return AgentVerseReply {
                    task_id: task.id,
                    allowed: false,
                    quarantined: false,
                    reason: Some(format!("InfranetGuard denied packet: {}", reason)),
                    text: None,
                };
            }
            PolicyDecision::AllowWithConstraints { reason, redactions } => {
                // For CHAT flows, we treat constraints as hints when building Tsafe request.
                let answer = self.call_tsafegate(task, Some(redactions));
                return AgentVerseReply {
                    task_id: answer.0,
                    allowed: true,
                    quarantined: false,
                    reason: Some(reason),
                    text: answer.1,
                };
            }
            PolicyDecision::Allow => {
                let answer = self.call_tsafegate(task, None);
                return AgentVerseReply {
                    task_id: answer.0,
                    allowed: true,
                    quarantined: false,
                    reason: None,
                    text: answer.1,
                };
            }
        }
    }

    // ---------- 5. Packet builder: Agent‑Verse → Infranet --------------

    fn build_packet(&self, task: &AgentVerseTask) -> SovereignPacket {
        let subj = &task.identity.subject_bostrom;

        let src = SovereignAddress {
            subjectid: subj.clone(),
            ocpuid: None,
        };
        let dst = src.clone(); // local sovereign node for now.

        let route = InfranetRouteKind::GovernanceChat;

        let roh_slice = Some(RoHSlice {
            value: self.config.default_roh.value,
            rohbefore: self.config.default_roh.value,
            rohafter: self.config.default_roh.value,
            rohceiling: 0.3,
        });

        let neurorights_env = NeurorightsEnvelope {
            mentalprivacy: true,
            mentalintegrity: true,
            cognitiveliberty: true,
            noncommercialneuraldata: true,
            dreamstatesensitive: false,
            forbiddecisionuse: false,
        };

        let capability = CapabilityScope {
            biophysicalscope: "ReadOnly".into(),
            actuationrights: "SuggestOnly".into(),
            safetyprofile: "MonotoneSafetyUpdate".into(),
            rightsprofile: "NeurorightsBound".into(),
        };

        SovereignPacket {
            src,
            dst,
            route,
            timestamp: SystemTime::now(),
            roh: roh_slice,
            neurorights: neurorights_env,
            tokenclass: task.identity.tokencategory(),
            capability,
            payloadtype: "AgentVersePrompt".into(),
            payloadref: format!("task:{}", task.id),
            hexstamp: None,
        }
    }

    // ---------- 6. Tsafe request builder + call -------------------------

    fn call_tsafegate(
        &self,
        task: AgentVerseTask,
        redactions: Option<Vec<String>>,
    ) -> (Uuid, Option<String>) {
        let subject = task.identity.subject_bostrom.clone();

        // CapabilityChord derived from role.
        let mut chord = CapabilityChord::new(
            task.identity.default_capability_kind(),
            subject.clone(),
        );
        chord.maxtokens = 2048;
        chord.actuationrights = "SuggestOnly".into(); // Agent‑Verse side is *always* non‑actuating.

        // Map task → XRAction; all Agent‑Verse flows here are modeled as CHAT.
        let action_kind = match task.kind {
            AgentVerseTaskKind::DraftPolicyText
            | AgentVerseTaskKind::ExplainShard
            | AgentVerseTaskKind::SuggestEvolveProposal
            | AgentVerseTaskKind::PlanXRRoute => XRActionKind::XRRouteStep,
        };

        let action = XRAction {
            kind: action_kind,
            subjectid: subject.clone(),
            route: "CHAT".into(),
            requestedfields: redactions.unwrap_or_default(),
            lifeforcecost: 0.0,
            rohbefore: self.config.default_roh.value,
            rohafterestimate: self.config.default_roh.value,
        };

        let req = TsafeRequest {
            subjectid: subject.clone(),
            route: "CHAT".into(),
            rawprompt: Some(task.prompt.clone()),
            action,
            capability: chord,
        };

        let gate = self.tsafe_gate.as_ref();
        match gate.authorizerequest(req) {
            AuthorizationResult::Authorized(auth) => {
                // In a full impl, you would now call an LLM through the Tsafe LlmClient
                // using auth.action + capability. Here we just echo allowed.
                let text = Some(format!(
                    "[Tsafe-allowed CHAT action {:?} for subject {}]",
                    auth.action.kind, subject
                ));
                (task.id, text)
            }
            AuthorizationResult::Rejected(reason) => {
                let txt = Some(format!(
                    "Tsafe Cortex Gate rejected request: {}",
                    reason.message
                ));
                (task.id, txt)
            }
        }
    }
}

// ---------- 7. Helper: build InfranetMeta for edge guards (optional) ----

impl AgentVerseAdapterConfig {
    pub fn to_infranet_meta(
        &self,
        identity: &AgentVerseIdentity,
    ) -> InfranetMeta {
        InfranetMeta {
            subjectbostrom: identity.subject_bostrom.clone(),
            hostocpuid: "neuro-pc-local".into(),
            actuationclass: identity.route_actuation_class(),
            tokenscope: crate::infranet_core::packet::InfranetTokenScope {
                hassmart: matches!(identity.role, AgentVerseRole::SystemOrchestrator),
                hasevolve: false,
                haschat: true,
            },
            protections: vec![
                NeuralProtection::AuraBoundaryGuard,
                NeuralProtection::SoulNonTradeableShield,
                NeuralProtection::DreamSanctumFilter,
                NeuralProtection::BioLoadThrottle,
                NeuralProtection::SovereignKernelLock,
                NeuralProtection::InfranetSovereignMeshGuard,
                NeuralProtection::RoHMeshCeiling,
                NeuralProtection::NeuroIPTransitShield,
                NeuralProtection::SovereignJurisdictionCapsule,
            ],
            rohslice: self.default_roh.clone(),
            jurisdiction: self.phoenix_region_capsule.clone(),
            neuroip: true,
        }
    }
}
