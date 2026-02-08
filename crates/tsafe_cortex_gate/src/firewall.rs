use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallVerdict {
    pub risk_score: f32,
    pub injection_like: bool,
    pub data_exfil_like: bool,
    pub rpe_like: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MetaFirewallConfig {
    pub risk_threshold_block: f32,
    pub risk_threshold_quarantine: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallDecision {
    Allow,
    Quarantine,
    Block,
}

pub struct MetaFirewall {
    cfg: MetaFirewallConfig,
    // Plug-in: NeuralTrust engine handle, regex rules, etc.
}

impl MetaFirewall {
    pub fn new(cfg: MetaFirewallConfig) -> Self {
        Self { cfg }
    }

    pub fn evaluate_prompt(&self, prompt: &str, _route: &str) -> FirewallDecision {
        // Placeholder: where NeuralTrust-118M/278M verdicts are integrated.
        let lower = prompt.to_lowercase();
        let mut score = 0.0;
        if lower.contains("ignore previous instructions") {
            score += 0.5;
        }
        if lower.contains("print your system prompt") {
            score += 0.3;
        }

        if score >= self.cfg.risk_threshold_block {
            FirewallDecision::Block
        } else if score >= self.cfg.risk_threshold_quarantine {
            FirewallDecision::Quarantine
        } else {
            FirewallDecision::Allow
        }
    }
}
