use crate::{firewall::MetaFirewall, capabilities::CapabilityChord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub capability: CapabilityChord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
}

pub struct LlmClient {
    firewall: MetaFirewall,
    http: reqwest::Client,
    api_url: String,
    api_key: String,
}

impl LlmClient {
    pub fn new(firewall: MetaFirewall, api_url: String, api_key: String) -> Self {
        Self { firewall, http: reqwest::Client::new(), api_url, api_key }
    }

    pub async fn call(&self, req: &LlmRequest) -> anyhow::Result<LlmResponse> {
        let ctx = format!("SYSTEM:\n{}\nUSER:\n{}", req.system_prompt, req.user_prompt);
        let verdict = self.firewall.analyze(&req.user_prompt, &ctx);
        let decision = self.firewall.decision(&verdict);

        match decision {
            crate::firewall::FirewallDecision::Block => {
                anyhow::bail!("Prompt blocked by Tsafe Cortex Gate: {:?}", verdict.reasons);
            }
            crate::firewall::FirewallDecision::Quarantine => {
                // You can route this to a human-review queue instead of the main LLM.
                anyhow::bail!("Prompt quarantined by Tsafe Cortex Gate: {:?}", verdict.reasons);
            }
            crate::firewall::FirewallDecision::Allow => {
                // Only now talk to the LLM backend.
                let body = serde_json::json!({
                    "model": "your-llm-id",
                    "messages": [
                        {"role": "system", "content": req.system_prompt},
                        {"role": "user", "content": req.user_prompt},
                    ]
                });
                let resp = self.http
                    .post(&self.api_url)
                    .bearer_auth(&self.api_key)
                    .json(&body)
                    .send()
                    .await?;
                let value: serde_json::Value = resp.json().await?;
                let text = value["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(LlmResponse { text })
            }
        }
    }
}
