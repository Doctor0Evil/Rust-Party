use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    pub ja3: String,
    pub alpn: Vec<String>,
    pub cipher_suites: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEvent {
    pub host: String,
    pub path: String,
    pub method: String,
    pub status: u16,
    pub referer: Option<String>,
    pub user_agent: String,
    pub h2_stream_id: Option<u32>,
    pub tls_fingerprint: TlsFingerprint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub provider: String,            // "google" | "facebook"
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub code: Option<String>,
    pub events: Vec<HttpEvent>,      // ordered by time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Call {
    pub bucket: String,
    pub key_prefix: String,
    pub operation: String, // "ListObjectsV2" | "GetObject" | ...
    pub user_agent: String,
    pub referer: Option<String>,
    pub h2_stream_id: Option<u32>,
    pub tls_fingerprint: TlsFingerprint,
}
