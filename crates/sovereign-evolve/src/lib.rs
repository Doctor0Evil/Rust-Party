//! Sovereign-evolve: streaming guardian for .evolve.jsonl / .evolve.ndjson.
//! Enforces schema validation and context boundaries before prompt building.

use serde::de::DeserializeOwned;
use serde_json::Value;
use sovereign_core::{ChatRequest, EvolutionProposal, HexStamp};
use std::io::{BufRead, BufReader, Read};

/// Parsed line with associated hexstamp and raw JSON value for auditing.
#[derive(Debug, Clone)]
pub struct EvolveRecord {
    pub proposal: EvolutionProposal,
    pub raw: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum EvolveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Line too long")]
    LineTooLong,
    #[error("Empty line")]
    EmptyLine,
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
}

/// Maximum allowed line length for a single JSONL record.
const MAX_LINE_LEN: usize = 16 * 1024;

/// Streaming parser for .evolve.jsonl.
pub struct EvolveStreamGuard<R> {
    reader: BufReader<R>,
}

impl<R: Read> EvolveStreamGuard<R> {
    pub fn new(inner: R) -> Self {
        Self {
            reader: BufReader::new(inner),
        }
    }

    /// Iterate over validated EvolutionProposal records.
    pub fn records(mut self) -> impl Iterator<Item = Result<EvolveRecord, EvolveError>> {
        std::iter::from_fn(move || {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => None,
                Ok(_) => {
                    if line.trim().is_empty() {
                        return Some(Err(EvolveError::EmptyLine));
                    }
                    if line.len() > MAX_LINE_LEN {
                        return Some(Err(EvolveError::LineTooLong));
                    }
                    Some(parse_evolve_line(&line))
                }
                Err(e) => Some(Err(EvolveError::Io(e))),
            }
        })
    }
}

fn parse_evolve_line(line: &str) -> Result<EvolveRecord, EvolveError> {
    let raw: Value = serde_json::from_str(line)?;
    let proposal: EvolutionProposal = from_value_strict(raw.clone())?;
    Ok(EvolveRecord { proposal, raw })
}

fn from_value_strict<T: DeserializeOwned>(v: Value) -> Result<T, EvolveError> {
    let t: T = serde_json::from_value(v.clone())?;
    Ok(t)
}

/// DefensiveTokens-aware prompt builder.
/// This does not call any model; it only constructs a guarded ChatRequest.
pub fn build_guarded_chat_request(
    proposal: &EvolutionProposal,
    base_prompt: &str,
    route: &str,
    defensive_tokens: &[String],
) -> ChatRequest {
    // Strict, deterministic composition: DefensiveTokens first, then proposal kind, then prompt.
    let mut prompt = String::new();
    for tok in defensive_tokens {
        prompt.push_str(tok);
        prompt.push(' ');
    }
    prompt.push_str("[evolve:");
    prompt.push_str(&format!("{:?}", proposal.kind));
    prompt.push_str("] ");
    prompt.push_str(base_prompt);

    ChatRequest {
        answer_id: proposal.proposal_id.clone(),
        prompt,
        route: route.into(),
    }
}

/// Utility for constructing a HexStamp from raw bytes (e.g., SHA-256 digest).
pub fn hexstamp_from_bytes(bytes: &[u8]) -> HexStamp {
    HexStamp(hex::encode(bytes))
}
