use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidConfig {
    pub subject_did: String,          // host DID
    pub citizen_class: String,        // e.g. "CITIZEN", "LAB_ONLY"
    pub jurisdiction_capsule: String, // e.g. "phoenix-az-us"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub route: String,         // "XR", "NANOSWARM", "BCI"
    pub purpose: String,       // "THERAPY", "RESEARCH", etc.
    pub allow_actuation: bool, // data-only vs actuation allowed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentConfig {
    pub version: String,
    pub scopes: Vec<ConsentScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenStake {
    pub stake_points: u64,             // CITIZEN stake weight
    pub roles: Vec<String>,            // "Host", "OrganicCPU"
    pub veto_powers: Vec<String>,      // e.g. "BLOCK_EVOLVE_NANOSWARM"
    pub max_evolve_rate_per_day: f32,  // structural evolution budget
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatic {
    pub did: DidConfig,
    pub consent: ConsentConfig,
    pub citizen_stake: CitizenStake,
    pub attributes: HashMap<String, String>, // free-form tags
}

impl GovernanceStatic {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let did: DidConfig =
            serde_json::from_str(&std::fs::read_to_string(dir.join("did.json"))?)?;
        let consent: ConsentConfig =
            serde_json::from_str(&std::fs::read_to_string(dir.join("consent.json"))?)?;
        let citizen_stake: CitizenStake =
            serde_json::from_str(&std::fs::read_to_string(dir.join("citizen-stake.json"))?)?;
        let attributes = HashMap::new();
        Ok(Self { did, consent, citizen_stake, attributes })
    }

    pub fn allows_actuation_route(&self, route: &str) -> bool {
        self.consent.scopes.iter().any(|s| s.route == route && s.allow_actuation)
    }
}
