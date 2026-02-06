use aura_boundary_guard::{AuraBoundaryGuard, CapabilityChord, CapabilityKind};
use bio_load_throttle::BioLoadThrottle;
use soul_non_tradeable_shield::{SoulNonTradeableShield, FsOpContext, FsOpKind};

pub struct TsafeCortexGate<GNet, GFs> {
    pub aura: AuraBoundaryGuard,
    pub bio: BioLoadThrottle,
    pub shield: SoulNonTradeableShield,
    pub net_client: GNet,
    pub vfs: GFs,
}

impl<GNet, GFs> TsafeCortexGate<GNet, GFs> {
    pub fn filter_capabilities(
        &self,
        all: &[CapabilityKind],
    ) -> anyhow::Result<Vec<CapabilityKind>> {
        Ok(self.bio.get_available_capabilities(all)?)
    }

    pub fn guarded_open(&self, ctx: &FsOpContext) -> anyhow::Result<()> {
        self.shield.check(ctx)?;
        Ok(())
    }

    pub fn guarded_http_get(
        &mut self,
        cap: &CapabilityChord,
        url: &str,
    ) -> anyhow::Result<reqwest::blocking::Response> {
        self.aura
            .can_access_network(cap, url::Url::parse(url)?.host_str().unwrap(), 443, None)?;
        Ok(reqwest::blocking::get(url)?)
    }
}
