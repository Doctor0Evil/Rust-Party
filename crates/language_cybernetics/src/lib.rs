use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Route and context for this cybernetic dialogue (BCI, OTA, GOV, CHAT, FS-EXPERIMENT, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyberRoute {
    Bci,
    Ota,
    Governance,
    Chat,
    FsExperiment,
    Research,
}

/// Minimal 5D cybernetic context for this turn (Reality.OS state slice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberContext5D {
    pub biostate: String,     // e.g. "calm", "fatigued-low", "overloaded"
    pub neurostate: String,   // e.g. "analysis", "dreamlike", "coding"
    pub lifeforce: String,    // e.g. "normal", "low", "critical"
    pub context: String,      // e.g. "reality.os.fs-lab", "xr-grid"
    pub sovereignty: String,  // e.g. "bostrom-primary", "lab-sandbox"
}

/// High-level cybernetic intent for research and experimental FS design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyberIntent {
    /// Ask the LLM to expand theory: feedback loops, RoH, neurorights, 5D modes, etc.
    ResearchTheory,
    /// Ask for experimental neuromorph FS designs (neuro-eXpFS / NeuroXFS variants).
    DesignFsExperiment,
    /// Ask for runtime policies and guards (AuraBoundaryGuard, BioLoadThrottle, etc.).
    DesignGuards,
    /// Ask for Reality.OS integration patterns (control plane, Tsafe Cortex Gate, etc.).
    IntegrateRealityOs,
}

/// What the LLM is allowed to do on this call (capability chord).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyberCapabilityKind {
    SummarizeCybernetics,
    ProposeFsExperiment,
    ProposePolicyGuard,
    DraftRealityOsModule,
}

/// Short-lived capability descriptor for this AI turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberCapability {
    pub id: Uuid,
    pub kind: CyberCapabilityKind,
    pub subject_id: String,    // Bostrom / OrganicCPU subject
    pub maxtokens: u32,
    pub expires_at_unix: i64,  // short-lived, e.g. now + 60s
}

/// How AI-chat should structure its answer so Reality.OS can act safely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberPromptFrame {
    /// System: non-negotiable rules (never executed directly, only interpreted by Rust).
    pub system_instructions: String,
    /// Human-facing question: what you are asking about cybernetics.
    pub user_question: String,
    /// Route: which Reality.OS channel this belongs to.
    pub route: CyberRoute,
    /// 5D slice: biophysical / sovereignty context for this interaction.
    pub ctx5d: CyberContext5D,
    /// Cybernetic intent: what type of knowledge / design you want.
    pub intent: CyberIntent,
    /// Capability chord: what the model is allowed to do (speak-only).
    pub capability: CyberCapability,
}

/// Structured output that Reality.OS will accept from AI-chat.
/// Everything is "speak-only": the kernel will review, filter, and apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberAnswer {
    /// Natural language explanation, for you (research / intuition).
    pub narrative: String,
    /// Optional list of experimental FS operations as *proposals*,
    /// never commands. Reality.OS will filter and apply or discard.
    pub fs_experiments: Vec<ExperimentalFsOp>,
    /// Optional list of new cybernetic concepts / virtual objects discovered.
    pub new_concepts: Vec<CyberConcept>,
}

/// A single proposed experimental filesystem operation in a neuromorph FS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalFsOp {
    pub op_id: Uuid,
    /// e.g. "create_shard_class", "attach_guard", "define_5d_mode"
    pub op_kind: String,
    /// Path or logical name inside neuro-eXpFS / Reality.OS FS space.
    pub target: String,
    /// Purely descriptive payload; kernel will map to real actions if allowed.
    pub description: String,
    /// Risk-of-Harm estimate in [0.0, 1.0] for research classification.
    pub roh_estimate: f32,
}

/// A new virtual object or cybernetic construct discovered in this dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberConcept {
    pub name: String,
    pub description: String,
    /// Optional mapping into your 5D space (which axes it touches).
    pub axes: Vec<String>, // e.g. ["lifeforce", "governance"]
}

/// Main function: build a language-cybernetics frame that Reality.OS
/// can send to the LLM, and that will come back as a CyberAnswer.
///
/// This is the core “language_cybernetics” primitive: it turns free-form
/// curiosity into a structured cybernetic experiment with explicit
/// boundaries and FS-awareness.
pub fn build_language_cybernetics_frame(
    route: CyberRoute,
    ctx5d: CyberContext5D,
    intent: CyberIntent,
    subject_id: String,
    user_question: String,
) -> CyberPromptFrame {
    use chrono::Utc;

    let capability_kind = match intent {
        CyberIntent::ResearchTheory => CyberCapabilityKind::SummarizeCybernetics,
        CyberIntent::DesignFsExperiment => CyberCapabilityKind::ProposeFsExperiment,
        CyberIntent::DesignGuards => CyberCapabilityKind::ProposePolicyGuard,
        CyberIntent::IntegrateRealityOs => CyberCapabilityKind::DraftRealityOsModule,
    };

    let capability = CyberCapability {
        id: Uuid::new_v4(),
        kind: capability_kind,
        subject_id,
        maxtokens: 2048,
        expires_at_unix: Utc::now().timestamp() + 60,
    };

    let system_instructions = r#"
You are a *language cybernetics* unit inside Reality.OS.
You NEVER execute actions. You ONLY:

- Expand and refine cybernetics knowledge: feedback loops, RoH, neurorights, 5D modes, neuromorph FS.
- Propose experimental filesystem operations as *descriptive JSON objects* (ExperimentalFsOp), never as shell commands.
- Propose new cybernetic concepts / virtual objects (CyberConcept) that can be mapped into a 5D biophysical space.
- Respect neurorights: no raw .neuroaln, no keys, no .bchainproof.json, no direct OTA or BCI control.
- Treat the OrganicCPU and Bostrom identity as sovereign; your output is advisory only.

Always answer as a `CyberAnswer` JSON object.
"#.to_string();

    CyberPromptFrame {
        system_instructions,
        user_question,
        route,
        ctx5d,
        intent,
        capability,
    }
}
