use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeAxis {
    pub name: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeKernel {
    pub axes: Vec<TsafeAxis>,
    // existing fields: constraints, tags, etc.
}

impl TsafeKernel {
    pub fn get_axis_bounds(&self, name: &str) -> Option<(f32, f32)> {
        self.axes
            .iter()
            .find(|a| a.name == name)
            .map(|a| (a.min, a.max))
    }
}
