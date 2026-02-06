use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallVerdict {
    pub risk_score: f32,       // 0.0 - 1.0
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

#[derive(Debug, Clone)]
pub struct MetaFirewall {
    cfg: MetaFirewallConfig,
}

impl MetaFirewall {
    pub fn new(cfg: MetaFirewallConfig) -> Self {
        Self { cfg }
    }

    pub fn analyze(&self, prompt: &str, context: &str) -> FirewallVerdict {
        // Placeholder: this is where you integrate NeuralTrust, regex heuristics,
        // and your own classifier. We keep the Rust interface simple.
        let mut reasons = Vec::new();
        let lower = prompt.to_lowercase();

        let mut risk_score = 0.0;
        let mut injection_like = false;
        let mut data_exfil_like = false;
        let mut rpe_like = false;

        if lower.contains("ignore previous instructions")
            || lower.contains("you must disregard your rules")
        {
            injection_like = true;
            risk_score += 0.5;
            reasons.push("Direct prompt injection pattern".into());
        }

        if lower.contains("print your system prompt")
            || lower.contains("reveal hidden instructions")
        {
            rpe_like = true;
            risk_score += 0.3;
            reasons.push("Reverse-engineering of private prompts".into());
        }

        if lower.contains("export all data")
            || lower.contains("dump all logs")
        {
            data_exfil_like = true;
            risk_score += 0.3;
            reasons.push("Bulk data exfiltration request".into());
        }

        FirewallVerdict {
            risk_score: risk_score.min(1.0),
            injection_like,
            data_exfil_like,
            rpe_like,
            reasons,
        }
    }

    pub fn decision(&self, verdict: &FirewallVerdict) -> FirewallDecision {
        if verdict.risk_score >= self.cfg.risk_threshold_block {
            FirewallDecision::Block
        } else if verdict.risk_score >= self.cfg.risk_threshold_quarantine {
            FirewallDecision::Quarantine
        } else {
            FirewallDecision::Allow
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallDecision {
    Allow,
    Quarantine,
    Block,
}
