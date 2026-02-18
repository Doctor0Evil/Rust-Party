use serde::{Deserialize, Serialize};
use sovereign_kernel::types::{GuardError, GuardResult, InfranetGuard, SovereignAction, SovereignActionKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionCapsule {
    pub region: String,
    pub allow_cross_border: bool,
}

pub struct InfranetGuardImpl {
    pub capsule: JurisdictionCapsule,
}

impl InfranetGuard for InfranetGuardImpl {
    fn validate_packet(&self, action: &SovereignAction) -> GuardResult {
        if !matches!(action.kind, SovereignActionKind::InfranetPacket) {
            return Ok(());
        }

        // Example: disallow cross-border high-risk routes if forbidden.
        if !self.capsule.allow_cross_border && action.route == "OTA" {
            return Err(GuardError {
                code: "INFRA_JURISDICTION".into(),
                message: "Cross-border OTA not allowed by JurisdictionCapsule.".into(),
            });
        }

        Ok(())
    }
}
