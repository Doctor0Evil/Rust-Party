use sovereign_specs::evolve_ledger::EvolutionProposalRecord;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> anyhow::Result<()> {
    let path = "policies/.evolve.jsonl";
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: EvolutionProposalRecord = serde_json::from_str(&line)?;
        if let Err(e) = record.validate_invariants() {
            eprintln!("Evolution record {} failed invariants: {}", idx, e);
            std::process::exit(1);
        }
    }

    Ok(())
}
