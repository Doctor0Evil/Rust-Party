use crate::packet::InfranetMeta;
use crate::protections::{NeuralProtection, RouteActuationClass};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardDecision {
    Allow,
    Throttle { reason: String },
    Drop { reason: String },
}

pub trait InfranetEdgeGuard {
    fn check_mesh_guard(&self, meta: &InfranetMeta) -> GuardDecision;
    fn check_roh_mesh_ceiling(&self, meta: &InfranetMeta, path_roh_max: f32) -> GuardDecision;
    fn check_neuro_ip_transit(&self, meta: &InfranetMeta) -> GuardDecision;
    fn check_jurisdiction_capsule(&self, meta: &InfranetMeta) -> GuardDecision;
}

pub struct DefaultEdgeGuard {
    pub roh_ceiling: f32, // e.g. 0.3
}

impl InfranetEdgeGuard for DefaultEdgeGuard {
    fn check_mesh_guard(&self, meta: &InfranetMeta) -> GuardDecision {
        if meta.protections.contains(&NeuralProtection::InfranetSovereignMeshGuard) {
            match meta.actuation_class {
                RouteActuationClass::Actuating => {
                    if !meta.token_scope.has_evolve {
                        return GuardDecision::Drop {
                            reason: "Actuating route without EVOLVE token".into(),
                        };
                    }
                }
                _ => {}
            }
        }
        GuardDecision::Allow
    }

    fn check_roh_mesh_ceiling(&self, meta: &InfranetMeta, path_roh_max: f32) -> GuardDecision {
        let candidate = meta.roh_slice.value.max(path_roh_max);
        if candidate > self.roh_ceiling || !meta.roh_slice.monotone_ok {
            return GuardDecision::Drop {
                reason: "RoHMeshCeiling violated on route".into(),
            };
        }
        GuardDecision::Allow
    }

    fn check_neuro_ip_transit(&self, meta: &InfranetMeta) -> GuardDecision {
        if meta.neuro_ip && meta.protections.contains(&NeuralProtection::NeuroIPTransitShield) {
            // Enforce noncommercial, non-exportable semantics at the edge.
            // Implementation details depend on peer classification, but default is to drop.
            GuardDecision::Drop {
                reason: "NeuroIPTransitShield forbids export to this domain".into(),
            }
        } else {
            GuardDecision::Allow
        }
    }

    fn check_jurisdiction_capsule(&self, meta: &InfranetMeta) -> GuardDecision {
        if meta
            .protections
            .contains(&NeuralProtection::SovereignJurisdictionCapsule)
            && meta.jurisdiction.lab_only
        {
            // Strictest-wins: lab-only capsules never forward to public networks.
            GuardDecision::Drop {
                reason: "SovereignJurisdictionCapsule: lab-only route".into(),
            }
        } else {
            GuardDecision::Allow
        }
    }
}
