use hd5d_core::{Hypervector, Identity5D, IdentityEncoder};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphicEvent {
    pub x: u16,
    pub y: u16,
    pub polarity: bool,
    pub timestamp_us: u64,
}

pub struct EventHdEncoder {
    dim: usize,
}

impl EventHdEncoder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn encode_spatial(&self, x: u16, y: u16) -> Hypervector {
        // Placeholder: in real EventHD, encode via structured spatial scheme.
        Hypervector::random()
    }

    fn encode_temporal(&self, t_us: u64) -> Hypervector {
        // Placeholder: encode coarse time bucket to hypervector.
        Hypervector::random()
    }

    pub fn encode_event(&self, ev: &NeuromorphicEvent) -> Hypervector {
        let spatial = self.encode_spatial(ev.x, ev.y);
        let temporal = self.encode_temporal(ev.timestamp_us);
        spatial.bind(&temporal)
    }

    pub fn encode_window(
        &self,
        events: &[NeuromorphicEvent],
        id5d: &Identity5D,
        id_encoder: &IdentityEncoder,
    ) -> Hypervector {
        let event_hvs: Vec<Hypervector> = events.iter().map(|e| self.encode_event(e)).collect();
        let events_superposed = Hypervector::superpose(&event_hvs);
        let id_hv = id_encoder.encode(id5d);
        events_superposed.bind(&id_hv)
    }
}
