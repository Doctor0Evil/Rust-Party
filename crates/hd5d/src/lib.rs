use rand::Rng;

/// Simple binary hypervector for hyperdimensional computing.
/// Dimension is fixed at compile time for now.
pub const DIM: usize = 10_000;

#[derive(Clone)]
pub struct Hypervector {
    pub bits: Vec<bool>,
}

impl Hypervector {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let bits = (0..DIM).map(|_| rng.gen::<bool>()).collect();
        Self { bits }
    }

    /// Element-wise XOR binding.
    pub fn bind(&self, other: &Hypervector) -> Hypervector {
        let bits = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .map(|(a, b)| *a ^ *b)
            .collect();
        Hypervector { bits }
    }

    /// Superposition by majority vote over a slice of hypervectors.
    pub fn superpose(vectors: &[Hypervector]) -> Hypervector {
        let mut counts = vec![0i32; DIM];
        for hv in vectors {
            for (i, bit) in hv.bits.iter().enumerate() {
                counts[i] += if *bit { 1 } else { -1 };
            }
        }
        let bits = counts.into_iter().map(|c| c >= 0).collect();
        Hypervector { bits }
    }

    /// Normalized Hamming similarity (1.0 = identical, 0.0 = orthogonal).
    pub fn similarity(&self, other: &Hypervector) -> f32 {
        let same = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .filter(|(a, b)| a == b)
            .count();
        same as f32 / DIM as f32
    }
}

/// 5-D biophysical identity coordinate (coarse).
#[derive(Clone, Debug)]
pub struct Identity5D {
    pub bio_state: String,
    pub neuro_state: String,
    pub lifeforce: String,
    pub context: String,
    pub sovereignty: String,
}

/// Encoder from Identity5D into a single hypervector using binding.
pub struct IdentityEncoder {
    axis_base: Hypervector,
    bio_base: Hypervector,
    neuro_base: Hypervector,
    lifeforce_base: Hypervector,
    context_base: Hypervector,
    sovereignty_base: Hypervector,
}

impl IdentityEncoder {
    pub fn new() -> Self {
        Self {
            axis_base: Hypervector::random(),
            bio_base: Hypervector::random(),
            neuro_base: Hypervector::random(),
            lifeforce_base: Hypervector::random(),
            context_base: Hypervector::random(),
            sovereignty_base: Hypervector::random(),
        }
    }

    fn encode_label(label: &str, seed: &Hypervector) -> Hypervector {
        // Very simple label encoding: hash characters via repeated binding.
        let mut hv = seed.clone();
        for b in label.bytes() {
            let mut char_hv = Hypervector::random();
            // Bias char_hv slightly based on byte value.
            for (i, bit) in char_hv.bits.iter_mut().enumerate().step_by(257) {
                *bit ^= (b as usize + i) % 2 == 0;
            }
            hv = hv.bind(&char_hv);
        }
        hv
    }

    pub fn encode(&self, id: &Identity5D) -> Hypervector {
        let bio = Self::encode_label(&id.bio_state, &self.bio_base);
        let neuro = Self::encode_label(&id.neuro_state, &self.neuro_base);
        let life = Self::encode_label(&id.lifeforce, &self.lifeforce_base);
        let ctx = Self::encode_label(&id.context, &self.context_base);
        let sov = Self::encode_label(&id.sovereignty, &self.sovereignty_base);

        let bound = self.axis_base
            .bind(&bio)
            .bind(&neuro)
            .bind(&life)
            .bind(&ctx)
            .bind(&sov);

        bound
    }
}
