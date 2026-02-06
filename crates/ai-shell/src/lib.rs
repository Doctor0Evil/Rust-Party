use anyhow::Result;
use cortex_gate::policy::{Decision, PolicyEngine, SovereignAction, SovereignActionKind};
use serde::{Deserialize, Serialize;

#[derive(Clone)]
pub struct AiShellConfig {
    pub api_url: String,
    pub api_key: String,
    pub subjectid: String,
}

pub struct AiShell {
    client: reqwest::Client,
    cfg: AiShellConfig,
    policy: PolicyEngine,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    pub content: String,
}

impl AiShell {
    pub async fn new(cfg: AiShellConfig, policy_dir: &std::path::Path) -> Result<Self> {
        let client = reqwest::Client::new();
        let policy = PolicyEngine::load_from_dir(policy_dir)?;
        Ok(Self { client, cfg, policy })
    }

    pub async fn chat(&self, user_prompt: &str) -> Result<String> {
        // Treat every chat as a sovereign action of type CHAT (no direct shard access).
        let action = SovereignAction {
            kind: SovereignActionKind::ProposeEvolve, // or a dedicated Chat route type
            subjectid: self.cfg.subjectid.clone(),
            route: "CHAT".into(),
            contextlabels: vec!["ai-shell".into()],
            requestedfields: vec![],
            lifeforcecost: 0.01,
        };

        match self.policy.evaluate(&action) {
            Decision::Deny { reason } => {
                return Err(anyhow::anyhow!(
                    "Tsafe denied AI-chat interaction: {}",
                    reason
                ));
            }
            Decision::AllowWithConstraints { reason, .. } => {
                tracing::warn!("Chat allowed with constraints: {}", reason);
            }
            Decision::Allow { .. } => { /* ok */ }
        }

        let req_body = ChatRequest {
            model: "gpt-5.1",
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: "You are a neurorights-aware, sovereign-safe assistant.",
                },
                ChatMessage {
                    role: "user",
                    content: user_prompt,
                },
            ],
        };

        let resp = self
            .client
            .post(&self.cfg.api_url)
            .bearer_auth(&self.cfg.api_key)
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;

        let parsed: ChatResponse = resp.json().await?;
        Ok(parsed.content)
    }
}
