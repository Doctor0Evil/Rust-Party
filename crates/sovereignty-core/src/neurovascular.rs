use serde::{Deserialize, Serialize};

/// Runtime neurovascular corridor variables injected from BioState / QPU telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurovascularCorridor {
    /// Normalized neurovascular resistance index (0.0 = baseline, 1.0 = max safe).
    pub resistance_index: f32,
    /// Normalized venular growth rate (0.0 = baseline; positive = growth, negative = pruning).
    pub venular_growth: f32,
    /// Confidence in these measurements (0.0..1.0).
    pub confidence: f32,
}

impl NeurovascularCorridor {
    /// Clamp fields into safe numeric ranges for guard use.
    pub fn clamped(self) -> Self {
        fn clamp01(x: f32) -> f32 {
            if x.is_nan() { 0.0 } else { x.max(0.0).min(1.0) }
        }
        Self {
            resistance_index: clamp01(self.resistance_index),
            venular_growth: self.venular_growth.max(-1.0).min(1.0),
            confidence: clamp01(self.confidence),
        }
    }

    /// Returns a multiplicative safety factor for effect-size envelopes.
    /// Higher resistance or venular growth → tighter bounds.
    pub fn envelope_factor(&self) -> f32 {
        let c = self.confidence;
        let r = self.resistance_index;
        let g = self.venular_growth.max(0.0); // only penalize positive growth
        // Base factor in [0.5, 1.0]; more load reduces the factor.
        let load = (0.5 * r + 0.5 * g) * c;
        (1.0 - 0.5 * load).max(0.5)
    }
}
