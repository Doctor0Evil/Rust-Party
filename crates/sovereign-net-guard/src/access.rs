use serde::{Deserialize, Serialize};
use crate::model::HttpEvent;
use crate::guards::{Finding, FindingKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessSessionPolicy {
    pub allowed_hosts: Vec<String>,
    pub access_cookie_name: String,
    pub max_idle_minutes: u32,
    pub forbid_localstorage_tokens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEvent {
    pub origin: String,
    pub key: String,
    pub value_len: usize,
    pub is_local_storage: bool,
}

pub fn detect_access_anomalies(
    policy: &AccessSessionPolicy,
    http_events: &[HttpEvent],
    storage_events: &[StorageEvent],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Cross-host reuse of Access cookies (rough heuristic)
    let mut host_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for ev in http_events {
        *host_map.entry(ev.host.clone()).or_default() += 1;
        if !policy.allowed_hosts.iter().any(|h| ev.host.ends_with(h)) {
            findings.push(Finding {
                kind: FindingKind::TlsFingerprintDrift,
                severity: 0.7,
                message: format!("Access session used on unexpected host {}", ev.host),
            });
        }
    }

    // 2. Token-like blobs in localStorage when forbidden
    if policy.forbid_localstorage_tokens {
        for se in storage_events {
            if se.is_local_storage && se.value_len > 256 {
                findings.push(Finding {
                    kind: FindingKind::OAuthRedirectLeak,
                    severity: 0.8,
                    message: format!(
                        "Suspicious large localStorage item at {} key={}",
                        se.origin, se.key
                    ),
                });
            }
        }
    }

    findings
}
