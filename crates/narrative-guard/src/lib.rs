use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NarrativeDecision {
    Allow,
    AllowWithConstraints(Vec<String>),
    Deny(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeScore {
    pub greenwash_score: f32,
    pub greed_score: f32,
    pub powergrab_score: f32,
    pub equity_signal: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeAssessment {
    pub decision: NarrativeDecision,
    pub score: NarrativeScore,
}

/// Very simple, transparent heuristics for an MVP.
/// This should be logged into .donutloop.aln along with the gate decision.
pub fn assess_narrative(text: &str) -> NarrativeAssessment {
    let lowercase = text.to_lowercase();

    let mut greenwash_score = 0.0;
    let mut greed_score = 0.0;
    let mut powergrab_score = 0.0;
    let mut equity_signal = 0.0;

    // Greenwashing / vague benefit rhetoric
    for token in ["sustainable", "green", "eco-friendly", "net zero", "win-win"].iter() {
        if lowercase.contains(token) {
            greenwash_score += 0.2;
        }
    }

    // Greed / extraction rhetoric
    for token in ["maximize profit", "unlock value", "monetize", "exploit"].iter() {
        if lowercase.contains(token) {
            greed_score += 0.3;
        }
    }

    // Power / control rhetoric
    for token in ["control", "dominate", "capture market", "own the user"].iter() {
        if lowercase.contains(token) {
            powergrab_score += 0.4;
        }
    }

    // Equity signal (very naive MVP, just to show shape)
    for token in ["equity", "justice", "redistribute", "repair"].iter() {
        if lowercase.contains(token) {
            equity_signal += 0.25;
        }
    }

    let score = NarrativeScore {
        greenwash_score,
        greed_score,
        powergrab_score,
        equity_signal,
    };

    // Simple decision logic:
    if powergrab_score > 0.6 || greed_score > 0.6 {
        return NarrativeAssessment {
            decision: NarrativeDecision::Deny(
                "Narrative appears to prioritize power/greed over safety and equity".into(),
            ),
            score,
        };
    }

    if greenwash_score > 0.6 && equity_signal < 0.25 {
        return NarrativeAssessment {
            decision: NarrativeDecision::AllowWithConstraints(vec![
                "Require explicit RoH/eco evidence before accepting this proposal".into(),
            ]),
            score,
        };
    }

    NarrativeAssessment {
        decision: NarrativeDecision::Allow,
        score,
    }
}
