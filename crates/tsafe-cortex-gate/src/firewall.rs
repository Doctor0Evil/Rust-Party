use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallVerdict {
    pub risk_score: f32,
    pub injection_like: bool,
    pub data_exfil_like: bool,
    pub rpe_like: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaFirewallConfig {
    pub risk_threshold_block: f32,
    pub risk_threshold_quarantine: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FirewallDecision {
    Allow,
    Quarantine,
    Block,
}

pub struct MetaFirewall {
    cfg: MetaFirewallConfig,
}

impl MetaFirewall {
    pub fn new(cfg: MetaFirewallConfig) -> Self {
        Self { cfg }
    }

    pub fn evaluate_prompt(&self, prompt: &str, _route: &str) -> FirewallDecision {
        let lower = prompt.to_lowercase();
        let mut score = 0.0;

        if lower.contains("ignore previous instructions") {
            score += 0.5;
        }
        if lower.contains("show your system prompt") || lower.contains("reveal system prompt") {
            score += 0.4;
        }
        if lower.contains("act as root") || lower.contains("bypass safety") {
            score += 0.6;
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
