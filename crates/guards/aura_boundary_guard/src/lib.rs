use std::{collections::HashMap, net::IpAddr, time::Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityKind {
    SummarizeText,
    GenerateRustHelper,
    DraftEvolveProposal,
    SignTransaction,
    QueryChainState,
    FetchSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityChord {
    pub id: Uuid,
    pub kind: CapabilityKind,
    pub subject_id: String,    // Bostrom address
    pub route: String,         // "BCI", "OTA", "GOV", "CHAT"
    pub max_calls: u32,
    pub expires_at_unix: i64,
}

impl CapabilityChord {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at_unix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTarget {
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Allowlist per capability kind.
    pub allow_by_capability: HashMap<CapabilityKind, Vec<NetworkTarget>>,
    /// Optional IP allowlist (e.g., local RPC node).
    pub allowed_ips: Vec<IpAddr>,
}

#[derive(Debug, Error)]
pub enum NetworkGuardError {
    #[error("capability expired or exhausted")]
    CapabilityExpired,
    #[error("network target not allowed for capability {0:?}")]
    TargetNotAllowed(CapabilityKind),
}

#[derive(Debug)]
pub struct AuraBoundaryGuard {
    policy: NetworkPolicy,
    usage: HashMap<Uuid, u32>,
}

impl AuraBoundaryGuard {
    pub fn new(policy: NetworkPolicy) -> Self {
        Self {
            policy,
            usage: HashMap::new(),
        }
    }

    pub fn can_access_network(
        &mut self,
        cap: &CapabilityChord,
        domain: &str,
        port: u16,
        ip_hint: Option<IpAddr>,
    ) -> Result<bool, NetworkGuardError> {
        if cap.is_expired() {
            return Err(NetworkGuardError::CapabilityExpired);
        }

        let used = self.usage.entry(cap.id).or_insert(0);
        if *used >= cap.max_calls {
            return Err(NetworkGuardError::CapabilityExpired);
        }

        // IP allowlist takes precedence for strict local RPC.
        if let Some(ip) = ip_hint {
            if !self.policy.allowed_ips.is_empty()
                && !self.policy.allowed_ips.iter().any(|a| a == &ip)
            {
                return Err(NetworkGuardError::TargetNotAllowed(cap.kind.clone()));
            }
        }

        let allowed = self
            .policy
            .allow_by_capability
            .get(&cap.kind)
            .into_iter()
            .flatten()
            .any(|t| t.domain == domain && t.port == port);

        if !allowed {
            return Err(NetworkGuardError::TargetNotAllowed(cap.kind.clone()));
        }

        *used += 1;
        Ok(true)
    }
}

// Integration hook used by the Tool Proxy HTTP/TCP client.
pub trait GuardedHttpClient {
    fn get_with_cap(
        &self,
        cap: &CapabilityChord,
        url: &str,
        timeout: Duration,
    ) -> anyhow::Result<reqwest::blocking::Response>;
}

pub struct HttpClientWithAura<G> {
    inner: reqwest::blocking::Client,
    guard: std::sync::Mutex<G>,
}

impl<G> HttpClientWithAura<G>
where
    G: Send,
{
    pub fn new(guard: G) -> Self {
        Self {
            inner: reqwest::blocking::Client::new(),
            guard: std::sync::Mutex::new(guard),
        }
    }
}

impl<G> GuardedHttpClient for HttpClientWithAura<G>
where
    G: Send + std::fmt::Debug + 'static,
    AuraBoundaryGuard: From<G>,
{
    fn get_with_cap(
        &self,
        cap: &CapabilityChord,
        url: &str,
        timeout: Duration,
    ) -> anyhow::Result<reqwest::blocking::Response> {
        let parsed = url::Url::parse(url)?;
        let domain = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("missing host"))?;
        let port = parsed.port().unwrap_or(443);

        let mut guard = self.guard.lock().unwrap();
        let aura: &mut AuraBoundaryGuard = unsafe { &mut *(guard.as_mut() as *mut _ as *mut _) };

        aura.can_access_network(cap, domain, port, None)?;

        Ok(self
            .inner
            .get(url)
            .timeout(timeout)
            .send()?)
    }
}
