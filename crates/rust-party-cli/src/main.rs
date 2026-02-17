use tsafe_cortex_gate::{
    TsafeCortexGate, DonutloopLogger,
    firewall::{MetaFirewall, MetaFirewallConfig},
    guardians::{RohGuard, NeurorightsGuard, EcoGuard},
    XRAction, XRActionKind, CapabilityChord, CapabilityKind, Request,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    // Load Tsafe policies from shards/root (NeuroPC local).
    let roh_guard = RohGuard::from_file("policies/rohmodel.aln")?;
    let nr_guard = NeurorightsGuard::from_file("policies/neurorights.json")?;
    let eco_guard = EcoGuard::from_file("policies/ecoenv.json")?;

    let firewall = MetaFirewall::new(MetaFirewallConfig {
        risk_threshold_block: 0.7,
        risk_threshold_quarantine: 0.4,
    });

    let donut = DonutloopLogger::new("shards/ledger/donutloop.aln");

    let gate = TsafeCortexGate::new(firewall, roh_guard, nr_guard, eco_guard, donut);

    // Example: safe, suggestion-only CHAT request, no actuation.
    let action = XRAction {
        kind: XRActionKind::XRRouteStep,
        subject_id: "bostrom18sd2u...".into(),
        route: "CHAT".into(),
        requested_fields: vec!["explain_roh_model".into()],
        lifeforce_cost: 0.01,
        roh_before: 0.12,
        roh_after_estimate: 0.12,
    };

    let cap = CapabilityChord {
        id: Uuid::new_v4(),
        kind: CapabilityKind::ExplainPolicy,
        subject_id: action.subject_id.clone(),
        max_tokens: 1024,
        expires_at_unix: chrono::Utc::now().timestamp() + 60,
        actuation_rights: "SuggestOnly".into(),
    };

    let req = Request {
        subject_id: action.subject_id.clone(),
        route: "CHAT".into(),
        raw_prompt: Some("Explain my current RoH model safely".into()),
        action,
        capability: cap,
    };

    match gate.authorize(req) {
        tsafe_cortex_gate::AuthorizationResult::Authorized(_) => {
            println!("AUTHORIZED: safe CHAT explanation");
        }
        tsafe_cortex_gate::AuthorizationResult::Rejected(reason) => {
            eprintln!("REJECTED: {} - {}", reason.code, reason.message);
        }
    }

    Ok(())
}
