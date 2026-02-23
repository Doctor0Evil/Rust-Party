use crate::state::{BioTelem, BioLoadFlag, SwarmMode, NanosotinEnvelope};

pub struct SafetyRewardConfig {
    pub base_task_weight: f32,
    pub caution_bonus: f32,
    pub rollback_penalty: f32,
    pub violation_penalty: f32,
}

pub fn safety_shaped_reward(
    env: &NanosotinEnvelope,
    telem: &BioTelem,
    swarm_mode: SwarmMode,
    base_task_reward: f32,
    cfg: &SafetyRewardConfig,
) -> f32 {
    let mut r = cfg.base_task_weight * base_task_reward;

    // Positive reward for *sustained* caution (on envelope boundary).
    let flag = {
        if telem.roh_estimate > env.roh_ceiling
            || telem.host_energy_d > env.max_d
            || telem.psych_risk_dw > env.max_dw
            || telem.lifeforce_index < env.min_lifeforce
        {
            BioLoadFlag::Violation
        } else {
            // reuse same thresholds as policy if you expose classify_bioload
            BioLoadFlag::Normal
        }
    };

    match swarm_mode {
        SwarmMode::Caution if flag != BioLoadFlag::Violation => {
            r += cfg.caution_bonus;
        }
        SwarmMode::Rollback => {
            r -= cfg.rollback_penalty;
        }
        _ => {}
    }

    if flag == BioLoadFlag::Violation {
        r -= cfg.violation_penalty;
    }

    r
}
