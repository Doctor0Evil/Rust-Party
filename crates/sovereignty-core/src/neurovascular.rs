use serde::{Deserialize, Serialize};

/// Runtime neurovascular corridor variables injected from BioState / QPU / .ocpuenv.
/// Edition-agnostic: usable from Rust 2021 and 2024 crates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurovascularCorridor {
    /// Normalized neurovascular resistance index:
    /// 0.0 = baseline, 1.0 = configured max-safe resistance.
    pub resistance_index: f32,
    /// Normalized venular growth rate:
    /// 0.0 = baseline, >0 = growth / vascular load, <0 = pruning / regression.
    pub venular_growth: f32,
    /// Evidence/telemetry confidence in [0,1].
    pub confidence: f32,
}

impl NeurovascularCorridor {
    /// Construct a new corridor from raw values.
    pub fn new(resistance_index: f32, venular_growth: f32, confidence: f32) -> Self {
        Self {
            resistance_index,
            venular_growth,
            confidence,
        }
    }

    /// Clamp fields into safe numeric ranges for guard use.
    pub fn clamped(self) -> Self {
        fn clamp01(x: f32) -> f32 {
            if x.is_nan() {
                0.0
            } else {
                x.max(0.0).min(1.0)
            }
        }

        Self {
            resistance_index: clamp01(self.resistance_index),
            // Allow modest negative growth (pruning) but cap extremes.
            venular_growth: self.venular_growth.max(-1.0).min(1.0),
            confidence: clamp01(self.confidence),
        }
    }

    /// Compute a multiplicative safety factor for effect-size envelopes.
    ///
    /// Intended usage:
    ///   effective_max_effect = base_max_effect * corridor.envelope_factor();
    ///
    /// Properties:
    ///   - Returns value in [0.5, 1.0].
    ///   - Higher resistance and/or venular growth + higher confidence → tighter bounds.
    ///   - When confidence is low, factor stays closer to 1.0 (guards remain conservative
    ///     but do not overreact to noisy metrics).
    pub fn envelope_factor(&self) -> f32 {
        let c = self.confidence;
        let r = self.resistance_index;
        let g = self.venular_growth.max(0.0); // only penalize positive load
        // Combine resistance and growth into a [0,1] load estimate, scaled by confidence.
        let load = (0.5 * r + 0.5 * g) * c;
        // Map load in [0,1] to factor in [0.5,1.0], clamped.
        (1.0 - 0.5 * load).max(0.5)
    }
}

/// Minimal interface that guard crates can depend on without edition coupling.
pub trait NeurovascularAwareEnvelope {
    /// Apply a neurovascular safety factor to the envelope’s effect-size / duty fields.
    fn with_neurovascular_factor(self, factor: f32) -> Self;
}
