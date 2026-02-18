use crate::{firewall::MetaFirewall, capabilities::CapabilityChord};

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Route class drives which DefensiveTokens profile (if any) is applied.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmRouteKind {
    BciControl,
    OtaGovernance,
    SovereignKernel,
    NeurorightsDraft,
    GeneralChat,
}

/// Security profile maps routes to token/strictness levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DefensiveProfile {
    High,   // BCI, OTA, sovereign kernel
    Medium, // neurorights drafting, policy text
    Off,    // low-stakes chat, experimentation
}

/// Minimal representation of a DefensiveTokens configuration for a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefensiveTokensConfig {
    /// Sequence of security tokens to inject at the *start* of the system prompt.
    pub prefix_tokens: Vec<String>,
    /// Optional suffix tokens (usually unused; kept for future experiments).
    pub suffix_tokens: Vec<String>,
}

/// LLM request is strictly structured (no freeform tools / shell).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub route: LlmRouteKind,
    /// Human-readable description for audit logs.
    pub purpose: String,
    /// System prompt body *without* DefensiveTokens.
    pub system_prompt: String,
    /// User content or input snippet.
    pub user_prompt: String,
    /// Timeout for the call.
    pub timeout: Duration,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
}

/// LLM response is always treated as a *proposal*, not a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The raw model text output, to be parsed by higher layers.
    pub text: String,
    /// For logging: which profile was used for this call.
    pub profile: DefensiveProfile,
}

/// Trait for a speak-only LLM client.
/// It is forbidden to execute any external IO or tools directly from responses.
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, req: LlmRequest) -> anyhow::Result<LlmResponse>;
}

/// Static policy that maps routes to profiles and token sets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSecurityPolicy {
    pub high_profile_tokens: DefensiveTokensConfig,
    pub medium_profile_tokens: DefensiveTokensConfig,
}

impl LlmSecurityPolicy {
    pub fn profile_for_route(route: LlmRouteKind) -> DefensiveProfile {
        match route {
            LlmRouteKind::BciControl
            | LlmRouteKind::OtaGovernance
            | LlmRouteKind::SovereignKernel => DefensiveProfile::High,
            LlmRouteKind::NeurorightsDraft => DefensiveProfile::Medium,
            LlmRouteKind::GeneralChat => DefensiveProfile::Off,
        }
    }

    pub fn tokens_for_profile(&self, profile: DefensiveProfile) -> Option<&DefensiveTokensConfig> {
        match profile {
            DefensiveProfile::High => Some(&self.high_profile_tokens),
            DefensiveProfile::Medium => Some(&self.medium_profile_tokens),
            DefensiveProfile::Off => None,
        }
    }

    /// Build the final system prompt with DefensiveTokens applied.
    pub fn build_system_prompt(
        &self,
        route: LlmRouteKind,
        base_system: &str,
    ) -> (DefensiveProfile, String) {
        let profile = Self::profile_for_route(route);
        if let Some(cfg) = self.tokens_for_profile(profile) {
            // Simple convention: join tokens with spaces before the system text.
            let mut composed = String::new();
            if !cfg.prefix_tokens.is_empty() {
                composed.push_str(&cfg.prefix_tokens.join(" "));
                composed.push('\n');
            }
            composed.push_str(base_system);
            if !cfg.suffix_tokens.is_empty() {
                composed.push('\n');
                composed.push_str(&cfg.suffix_tokens.join(" "));
            }
            (profile, composed)
        } else {
            (profile, base_system.to_owned())
        }
    }
}

/// A wrapper that enforces OWASP LLM01-style isolation:
/// - applies DefensiveTokens for high/medium routes,
/// - never exposes secrets or file paths,
/// - never executes responses (speak-only).
pub struct SecureLlmClient<C: LlmClient> {
    inner: C,
    policy: LlmSecurityPolicy,
}

impl<C: LlmClient> SecureLlmClient<C> {
    pub fn new(inner: C, policy: LlmSecurityPolicy) -> Self {
        Self { inner, policy }
    }

    /// High-level call: route-aware, DefensiveTokens-aware completion.
    pub async fn secure_complete(&self, mut req: LlmRequest) -> anyhow::Result<LlmResponse> {
        // Apply route-based system prompt hardening.
        let (profile, hardened_system) =
            self.policy.build_system_prompt(req.route, &req.system_prompt);
        req.system_prompt = hardened_system;

        // Delegate to the inner client; response is *still* only a proposal.
        let mut resp = self.inner.complete(req).await?;
        resp.profile = profile;
        Ok(resp)
    }
}
