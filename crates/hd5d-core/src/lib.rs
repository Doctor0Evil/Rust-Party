use rand::Rng;
use serde::{Deserialize, Serialize};

pub const DIM: usize = 10_000;

#[derive(Clone, Serialize, Deserialize)]
pub struct Hypervector {
    pub bits: Vec<bool>,
}

impl Hypervector {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let bits = (0..DIM).map(|_| rng.gen_bool(0.5)).collect();
        Self { bits }
    }

    pub fn bind(&self, other: &Hypervector) -> Hypervector {
        let bits = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .map(|(a, b)| *a ^ *b)
            .collect();
        Hypervector { bits }
    }

    pub fn superpose(vectors: &[Hypervector]) -> Hypervector {
        let mut counts = vec![0_i32; DIM];
        for hv in vectors {
            for (i, bit) in hv.bits.iter().enumerate() {
                counts[i] += if *bit { 1 } else { -1 };
            }
        }
        let bits = counts.into_iter().map(|c| c >= 0).collect();
        Hypervector { bits }
    }

    pub fn similarity(&self, other: &Hypervector) -> f32 {
        let same = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .filter(|(a, b)| *a == *b)
            .count() as f32;
        same / DIM as f32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identity5D {
    pub biostate: String,
    pub neurostate: String,
    pub lifeforce: String,
    pub context: String,
    pub sovereignty: String,
}

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
        // Simple label encoding: repeated binding of random char vectors.
        let mut hv = seed.clone();
        for _b in label.bytes() {
            let char_hv = Hypervector::random();
            hv = hv.bind(&char_hv);
        }
        hv
    }

    pub fn encode(&self, id: &Identity5D) -> Hypervector {
        let bio = Self::encode_label(&id.biostate, &self.bio_base);
        let neuro = Self::encode_label(&id.neurostate, &self.neuro_base);
        let life = Self::encode_label(&id.lifeforce, &self.lifeforce_base);
        let ctx = Self::encode_label(&id.context, &self.context_base);
        let sov = Self::encode_label(&id.sovereignty, &self.sovereignty_base);

        self.axis_base
            .bind(&bio)
            .bind(&neuro)
            .bind(&life)
            .bind(&ctx)
            .bind(&sov)
    }
}
