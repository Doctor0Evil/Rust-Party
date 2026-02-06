//! Sovereign-kernel: compiles injector prefixes, LLM params, and system tuning
//! into a single neurorights-aware SovereignConfig for Rust-Party / NeuroPC.

use serde::{Deserialize, Serialize};
use sovereign_types::{
    Envelopes, NeurorightsPolicy, RoHModel, RuntimeMetrics, SovereignError, WorkspaceRef, OcpuEnv,
};

/// Injector roles derived from injector-prefix::[--role:...].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectorRole {
    Host,
    Subject { subject_id: String },
    Installer,
}

/// Session configuration derived from injector-prefix lanes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SovereignSessionConfig {
    pub role: InjectorRole,
    pub auditing: bool,
    pub event_log: bool,
    pub pkg_safe_mode: bool,
    pub parallelize: bool,
    pub no_cache: bool,
    pub auto_clean: bool,
    pub auto_update_pkgs: bool,
    pub session_persist: bool,
    pub fallback_tolerant: bool,
    pub checksum_verify: bool,
    pub immutable: bool,
    pub update_lock: bool,
    pub night_mode: bool,
    pub static_ip: bool,
    pub cloud_mirror: bool,
    pub disable_telemetry: bool,
    pub low_power_mode: bool,
}

impl SovereignSessionConfig {
    /// Guard builder: enforce neurorights, RoH, and bioscale envelopes.
    pub fn build(
        desired: Self,
        nr: &NeurorightsPolicy,
        roh: &RoHModel,
        env: &OcpuEnv,
    ) -> Result<Self, SovereignError> {
        if !nr.permits_telemetry(desired.disable_telemetry) {
            return Err(SovereignError::NeurorightsViolation);
        }
        if desired.parallelize && env.max_threads_for_duty_cycle(0.0) <= 0.0 {
            return Err(SovereignError::BioEnvelopeViolation);
        }
        if roh.global_ceiling() > 0.3 {
            return Err(SovereignError::RoHModelInvalid);
        }
        Ok(desired)
    }
}

/// Neurorights-aware LLM generation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub stop: Option<String>,
    pub random_seed: Option<u64>,
    pub json_output: bool,
    pub markdown: bool,
    pub min_tokens: u32,
}

impl LlmParams {
    pub fn guarded(self, nr: &NeurorightsPolicy, roh: &RoHModel) -> Result<Self, SovereignError> {
        if self.temperature > 0.6 && nr.domain_is_sensitive() {
            return Err(SovereignError::RiskTooHigh);
        }
        if self.min_tokens < nr.min_explanation_tokens() {
            return Err(SovereignError::ConsentTooWeak);
        }
        if roh.global_ceiling() > 0.3 {
            return Err(SovereignError::RoHModelInvalid);
        }
        Ok(self)
    }
}

/// Network priority tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPriority {
    Low,
    Normal,
    High,
}

/// System tuning parameters for OTA / runtime jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTuning {
    pub batch_size: u32,
    pub max_length: u32,
    pub timeout_secs: u32,
    pub memory_limit_bytes: u64,
    pub thread_count: u32,
    pub network_priority: NetworkPriority,
    pub gpu_enable: bool,
    pub async_mode: bool,
}

impl SystemTuning {
    pub fn check_against_ocpu(
        self,
        env: &OcpuEnv,
        metrics: &RuntimeMetrics,
    ) -> Result<Self, SovereignError> {
        let allowed_threads = env.max_threads_for_duty_cycle(metrics.current_duty_cycle);
        if self.thread_count as f32 > allowed_threads {
            return Err(SovereignError::BioEnvelopeViolation);
        }
        if self.memory_limit_bytes > env.max_memory_bytes() {
            return Err(SovereignError::ResourceLimitExceeded);
        }
        Ok(self)
    }
}

/// Sovereign profiles (inject-profile[...] mapping).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileId {
    Default,
    ExpertScientificCodex,
    SecureSandbox,
    PlatinumAuditor,
    SovereignKernelBostrom,
}

/// Request object for constructing a sovereign configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SovereignConfigRequest {
    pub role: InjectorRole,
    pub session: SovereignSessionConfig,
    pub llm: LlmParams,
    pub tuning: SystemTuning,
    pub workspace_ref: WorkspaceRef,
    pub profile: ProfileId,
}

/// Fully constructed, neurorights-aware sovereign configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SovereignConfig {
    pub role: InjectorRole,
    pub session: SovereignSessionConfig,
    pub llm: LlmParams,
    pub tuning: SystemTuning,
    pub neurorights: NeurorightsPolicy,
    pub roh_model: RoHModel,
    pub envelopes: Envelopes,
    pub workspace_ref: WorkspaceRef,
    pub profile: ProfileId,
}

impl SovereignConfig {
    /// Top-level constructor: compiles prefixes, params, tuning into a safe config.
    pub fn new(request: SovereignConfigRequest) -> Result<Self, SovereignError> {
        let nr = load_neurorights(&request.workspace_ref)?;
        let roh = load_roh_model(&request.workspace_ref)?;
        let envelopes = load_envelopes(&request.workspace_ref)?;

        let session =
            SovereignSessionConfig::build(request.session, &nr, &roh, &envelopes.ocpuenv)?;
        let llm = request.llm.guarded(&nr, &roh)?;
        let tuning =
            request
                .tuning
                .check_against_ocpu(&envelopes.ocpuenv, &envelopes.runtime_metrics)?;

        Ok(SovereignConfig {
            role: request.role,
            session,
            llm,
            tuning,
            neurorights: nr,
            roh_model: roh,
            envelopes,
            workspace_ref: request.workspace_ref,
            profile: request.profile,
        })
    }
}

/// Stubbed loaders: in production, bind to ALN/JSONL parsers and filesystem.
fn load_neurorights(ws: &WorkspaceRef) -> Result<NeurorightsPolicy, SovereignError> {
    let _ = ws;
    // Replace with real loading from .neurorights.json.
    Ok(NeurorightsPolicy {
        mental_privacy: true,
        mental_integrity: true,
        cognitive_liberty: true,
        noncommercial_neural_data: true,
        soulnontradeable: true,
        dreamstate_sensitive: true,
        forbid_decision_use: false,
        min_explanation_tokens: 32,
        domain_sensitive: true,
        allow_disable_telemetry: false,
    })
}

fn load_roh_model(ws: &WorkspaceRef) -> Result<RoHModel, SovereignError> {
    let _ = ws;
    // Replace with real loading from .rohmodel.aln.
    Ok(RoHModel {
        global_ceiling: sovereign_types::RoH(0.3),
    })
}

fn load_envelopes(ws: &WorkspaceRef) -> Result<Envelopes, SovereignError> {
    let _ = ws;
    // Replace with real loading from .ocpuenv, .lifeforce.aln, etc.
    Ok(Envelopes {
        ocpuenv: OcpuEnv {
            max_threads: 8,
            max_memory_bytes: 2 * 1024 * 1024 * 1024,
            max_duty_cycle_percent: 80.0,
        },
        runtime_metrics: RuntimeMetrics {
            current_duty_cycle: 10.0,
            fatigue_index: 0.1,
            eco_impact: 0.1,
        },
    })
}
