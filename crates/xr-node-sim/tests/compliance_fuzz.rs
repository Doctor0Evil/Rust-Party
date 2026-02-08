use std::path::PathBuf;
use xr_node_sim::run_randomized_compliance_trial;

#[test]
fn xr_node_randomized_compliance_report() {
    // Assume policies/ holds lab-grade neurorights.json, rohmodel.aln, tsafe.aln
    let mut policies_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    policies_dir.push("../../policies");

    let report = run_randomized_compliance_trial(&policies_dir, 512)
        .expect("sim compliance trial must succeed");

    // Basic sanity: we should see at least some RoH and neurorights denials
    // when fuzzing over deliberately adversarial samples.
    assert!(report.total_samples == 512);
    assert!(
        report.denied_roh > 0,
        "no RoH violations were detected in fuzz run; check adversarial mix"
    );
    assert!(
        report.denied_neurorights > 0,
        "no neurorights violations were detected in fuzz run; check adversarial mix"
    );

    // Print a human-readable summary into CI logs.
    eprintln!("{}", report.to_text_summary());
}
