use std::{path::Path, time::SystemTime};

use serde::{Deserialize, Serialize};

use infranet_core::packet::{
    CapabilityScope, InfranetRouteKind, NeurorightsEnvelope, RoHSlice, SovereignAddress,
    SovereignPacket, TokenClass,
};
use infranet_guard::{InfranetGuard, PolicyDecision};
use infranet_firewalls::{FirewallEngine, FirewallVerdict, InfranetFirewall};

/// Minimal representation of a chat request coming from any AI platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIngress {
    /// Human- or AI-facing account label (Perplexity, LocalLLM, etc.).
    pub territory: String,
    /// Bostrom / safe-alt address that owns this Rust-Party settlement.
    pub subject_bostrom: String,
    /// Route label, usually "CHAT".
    pub route: String,
    /// Logical channel or thread id.
    pub channel: String,
    /// Plaintext prompt or message.
    pub prompt: String,
}

/// Normalized, neurorights-aware representation before hitting Tsafe / Infranet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIntent {
    pub ingress: ChatIngress,
    /// High-level capability this chat is allowed (SuggestOnly, DraftOnly, etc.).
    pub capability: CapabilityScope,
    /// Token class granted to this territory (CHAT, SMART, EVOLVE, None).
    pub token_class: TokenClass,
    /// Estimated RoH slice (from 0.0 to 0.3) for this interaction.
    pub roh: Option<RoHSlice>,
    /// Neurorights posture applied to this chat.
    pub neurorights: NeurorightsEnvelope,
}

/// High-level result after Tsafe and firewall decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDecision {
    pub ingress: ChatIngress,
    pub policy: PolicyDecision,
    pub firewall: FirewallVerdict,
    pub timestamp: SystemTime,
}

pub struct RustPartyGateway<F: FirewallEngine> {
    guard: InfranetGuard,
    firewall: InfranetFirewall<F>,
}

impl<F: FirewallEngine> RustPartyGateway<F> {
    /// Load neurorights / Tsafe policies from a Rust-Party sovereign workspace.
    pub fn load_from_workspace<P: AsRef<Path>>(
        policies_dir: P,
        firewall_engine: F,
    ) -> anyhow::Result<Self> {
        let guard = InfranetGuard::load_from_policies(policies_dir.as_ref())?;
        let firewall = InfranetFirewall::new(firewall_engine);
        Ok(Self { guard, firewall })
    }

    /// Map an ingress chat message into a governed ChatIntent.
    pub fn classify_intent(&self, ingress: ChatIngress) -> ChatIntent {
        // Default neurorights posture: strict for Rust-Party territory.
        let neurorights = NeurorightsEnvelope {
            mentalprivacy: true,
            mentalintegrity: true,
            cognitiveliberty: true,
            noncommercialneuraldata: true,
            dreamstatesensitive: true,
            forbiddecisionuse: true,
        };

        // For multi-territory settlement, treat all external chats as CHAT-only, SuggestOnly.
        let capability = CapabilityScope {
            biophysicalscope: "ReadOnly".into(),
            actuationrights: "SuggestOnly".into(),
            safetyprofile: "MonotoneSafetyUpdate".into(),
            rightsprofile: "NeurorightsBound".into(),
        };

        let token_class = TokenClass::Chat;

        // For plain text chat, RoH effects are at or near baseline.
        let roh = Some(RoHSlice {
            rohbefore: 0.0,
            rohafter: 0.0,
            rohceiling: 0.3,
        });

        ChatIntent {
            ingress,
            capability,
            token_class,
            roh,
            neurorights,
        }
    }

    /// Convert ChatIntent into a SovereignPacket understood by InfranetGuard / Tsafe.
    fn intent_to_packet(&self, intent: &ChatIntent) -> SovereignPacket {
        let src = SovereignAddress {
            subjectid: intent.ingress.subject_bostrom.clone(),
            ocpuid: None,
        };
        let dst = src.clone();

        let route = InfranetRouteKind::GovernanceChat;

        SovereignPacket {
            src,
            dst,
            route,
            timestamp: SystemTime::now(),
            roh: intent.roh.clone(),
            neurorights: intent.neurorights.clone(),
            tokenclass: intent.token_class,
            capability: intent.capability.clone(),
            payloadtype: "ChatFragment".into(),
            // payloadref is a logical key; the actual prompt should be stored
            // in a local, neurorights-bound shard, not sent to untrusted tools.
            payloadref: format!(
                "rust-party:{}:{}",
                intent.ingress.territory, intent.ingress.channel
            ),
            hexstamp: None,
        }
    }

    /// End-to-end decision: firewall + Tsafe policy for a single chat ingress.
    pub fn decide(&self, ingress: ChatIngress) -> ChatDecision {
        let intent = self.classify_intent(ingress.clone());

        // Firewall operates on a key string; here we use territory + channel as label.
        let key = format!(
            "{}::{}",
            intent.ingress.territory, intent.ingress.channel
        );
        let firewall_verdict = self.firewall.evaluate_packet(self.intent_to_packet(&intent));

        // If firewall blocks, short-circuit policy as Deny with reason.
        let policy_decision = match firewall_verdict {
            FirewallVerdict::Block | FirewallVerdict::Quarantine => {
                PolicyDecision::Deny {
                    reason: "HostilityFenceFilter: firewall blocked or quarantined chat route"
                        .into(),
                }
            }
            FirewallVerdict::Allow => self.guard.evaluate(self.intent_to_packet(&intent)),
        };

        ChatDecision {
            ingress,
            policy: policy_decision,
            firewall: firewall_verdict,
            timestamp: SystemTime::now(),
        }
    }
}
