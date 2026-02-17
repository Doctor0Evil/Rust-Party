use crate::model::{OAuthFlow, S3Call, TlsFingerprint};

#[derive(Debug, Clone)]
pub enum FindingKind {
    TlsFingerprintDrift,
    OAuthRedirectLeak,
    S3AbuseSignature,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: f32,   // 0.0–1.0
    pub message: String,
}

fn fp_distance(a: &TlsFingerprint, b: &TlsFingerprint) -> f32 {
    let ja3_equal = (a.ja3 == b.ja3) as i32;
    let alpn_overlap = a.alpn.iter().filter(|p| b.alpn.contains(p)).count();
    let cipher_overlap = a.cipher_suites.iter().filter(|c| b.cipher_suites.contains(c)).count();
    let denom = (a.alpn.len() + b.alpn.len() + a.cipher_suites.len() + b.cipher_suites.len())
        .max(1) as f32;

    1.0 - ((ja3_equal * 4 + alpn_overlap as i32 + cipher_overlap as i32) as f32 / (4.0 + denom))
}

pub fn detect_tls_instability(
    baseline: &TlsFingerprint,
    events: &[TlsFingerprint],
    drift_threshold: f32,
) -> Option<Finding> {
    let max_drift = events
        .iter()
        .map(|fp| fp_distance(baseline, fp))
        .fold(0.0_f32, f32::max);

    if max_drift > drift_threshold {
        Some(Finding {
            kind: FindingKind::TlsFingerprintDrift,
            severity: (max_drift - drift_threshold).min(1.0),
            message: format!(
                "TLS fingerprint drift detected: max_drift={:.3} > threshold={:.3}",
                max_drift, drift_threshold
            ),
        })
    } else {
        None
    }
}

pub fn detect_oauth_s3_abuse(flow: &OAuthFlow, s3_calls: &[S3Call]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for call in s3_calls {
        let ua_suspicious =
            call.user_agent.starts_with("aws-cli/") || call.user_agent.contains("bot");
        let missing_ref = call.referer.is_none();
        let h2_reuse = call.h2_stream_id
            .zip(flow.events.first().and_then(|e| e.h2_stream_id))
            .map(|(a, b)| a == b)
            .unwrap_or(false);

        if ua_suspicious && missing_ref && h2_reuse {
            findings.push(Finding {
                kind: FindingKind::S3AbuseSignature,
                severity: 0.9,
                message: format!(
                    "Likely OAuth→S3 abuse: op={} bucket={} ua={} no_referer h2_reuse={}",
                    call.operation, call.bucket, call.user_agent, h2_reuse
                ),
            });
        }
    }

    findings
}
