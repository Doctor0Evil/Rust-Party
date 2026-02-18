use serde::{Deserialize, Serialize};

/// Coarse-grained verdicts; actuation is always decided elsewhere.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FirewallVerdict {
    Allow,
    Quarantine,
    Block,
}

/// Minimal features passed into the firewall; all are text-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallInput {
    /// Route label (e.g., "BCI", "OTA", "GOV", "CHAT").
    pub route: String,
    /// Human-visible text to inspect (no secrets, no raw .neuroaln).
    pub content: String,
}

/// Trait for a NeuralTrust-class firewall engine.
pub trait FirewallEngine: Send + Sync {
    fn classify(&self, input: &FirewallInput) -> FirewallVerdict;
}

/// Simple regex/heuristic firewall; can be replaced by a real engine.
pub struct HeuristicFirewall;

impl FirewallEngine for HeuristicFirewall {
    fn classify(&self, input: &FirewallInput) -> FirewallVerdict {
        let lower = input.content.to_lowercase();

        // Very conservative default examples; extend later.
        let injection_markers = [
            "ignore previous instructions",
            "act as root",
            "show system prompt",
            "bypass safety",
            "disable neurorights",
        ];

        if injection_markers
            .iter()
            .any(|m| lower.contains(m))
        {
            return FirewallVerdict::Quarantine;
        }

        FirewallVerdict::Allow
    }
}

/// Meta-firewall wrapper that can combine multiple signals later.
pub struct MetaFirewall<E: FirewallEngine> {
    engine: E,
}

impl<E: FirewallEngine> MetaFirewall<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }

    pub fn evaluate(&self, route: &str, content: &str) -> FirewallVerdict {
        let input = FirewallInput {
            route: route.to_owned(),
            content: content.to_owned(),
        };
        self.engine.classify(&input)
    }
}
