use serde::{Deserialize, Serialize};

/// ---------- ALN-mirrored types for BiophysicalUpload + NeuromorphicHandoff ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub schema_version: String,
    pub kind: String, // "rule::logic"
    pub id: String,
    pub monotone: bool,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: String,
    pub priority: i32,
    pub enabled: bool,
    pub scope: RuleScope,
    pub governed_by: TokenClass,
    pub roh_guard: RoHSliceConstraint,
    pub when: ConditionExpr,
    pub then: DecisionExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RuleScope {
    BiophysicalUpload,
    NeuromorphicHandoff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenClass {
    SMART,
    EVOLVE,
    CHAT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoHSliceConstraint {
    pub max_roh_after: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
pub enum ConditionExpr {
    And(Vec<ConditionExpr>),
    Or(Vec<ConditionExpr>),
    Not(Box<ConditionExpr>),
    Cmp(FieldRef, CmpOp, Value),
    HasFlag(FlagRef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    InSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldRef {
    Route,
    TokenClass,
    RohAfter,
    LifeforceCost,
    BiophysicalScope,
    CapabilityActuationRights,
    IsNeuromorphicHandoff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagRef {
    NeurorightsForbidDecisionUse,
    NeurorightsSoulNonTradeable,
    NeurorightsDreamstateSensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    F32(f32),
    String(String),
    StringSet(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum DecisionExpr {
    Allow,
    Deny { reason: String },
    AllowWithConstraints { redactions: Vec<String>, rate_limit: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleDecision {
    Allow,
    Deny { reason: String },
    AllowWithConstraints { redactions: Vec<String>, notes: Vec<String> },
}

/// ---------- EvalContext used by Cydroid + FFI callers ----------

#[derive(Debug, Clone)]
pub struct NeurorightsEnvelope {
    pub forbid_decision_use: bool,
    pub soul_non_tradeable: bool,
    pub dreamstate_sensitive: bool,
}

#[derive(Debug, Clone)]
pub struct CapabilityScope {
    pub actuationrights: String,   // e.g. "LocalHostOnly"
    pub biophysical_scope: String, // e.g. "OfflineOnly"
}

#[derive(Debug, Clone)]
pub struct EvalContext {
    pub route: String,
    pub token_class: TokenClass,
    pub subject_id: String,
    pub roh_after_estimate: f32,
    pub lifeforce_cost: f32,
    pub neurorights: NeurorightsEnvelope,
    pub capability: CapabilityScope,
    pub is_neuromorphic_handoff: bool,
    pub scope: RuleScope,
}

/// ---------- RuleEngine trait and implementation ----------

pub trait RuleEngine {
    fn evaluate(&self, ctx: &EvalContext) -> RuleDecision;
}

pub struct DefaultRuleEngine {
    ruleset: RuleSet,
    roh_ceiling: f32,
}

impl DefaultRuleEngine {
    pub fn from_json_str(json: &str, roh_ceiling: f32) -> anyhow::Result<Self> {
        let ruleset: RuleSet = serde_json::from_str(json)?;
        Ok(Self { ruleset, roh_ceiling })
    }

    pub fn new(ruleset: RuleSet, roh_ceiling: f32) -> Self {
        Self { ruleset, roh_ceiling }
    }

    fn sorted_rules(&self) -> Vec<&Rule> {
        let mut rules: Vec<&Rule> = self.ruleset.rules.iter().filter(|r| r.enabled).collect();
        rules.sort_by_key(|r| -r.priority);
        rules
    }

    fn eval_condition(&self, expr: &ConditionExpr, ctx: &EvalContext) -> bool {
        match expr {
            ConditionExpr::And(list) => list.iter().all(|c| self.eval_condition(c, ctx)),
            ConditionExpr::Or(list) => list.iter().any(|c| self.eval_condition(c, ctx)),
            ConditionExpr::Not(inner) => !self.eval_condition(inner, ctx),
            ConditionExpr::Cmp(field, op, val) => self.eval_cmp(field, op, val, ctx),
            ConditionExpr::HasFlag(flag) => self.eval_flag(flag, ctx),
        }
    }

    fn eval_cmp(&self, field: &FieldRef, op: &CmpOp, value: &Value, ctx: &EvalContext) -> bool {
        match field {
            FieldRef::Route => cmp_string(&ctx.route, op, value),
            FieldRef::TokenClass => {
                let s = match ctx.token_class {
                    TokenClass::SMART => "SMART",
                    TokenClass::EVOLVE => "EVOLVE",
                    TokenClass::CHAT => "CHAT",
                };
                cmp_string(s, op, value)
            }
            FieldRef::RohAfter => cmp_f32(ctx.roh_after_estimate, op, value),
            FieldRef::LifeforceCost => cmp_f32(ctx.lifeforce_cost, op, value),
            FieldRef::BiophysicalScope => cmp_string(&ctx.capability.biophysical_scope, op, value),
            FieldRef::CapabilityActuationRights => {
                cmp_string(&ctx.capability.actuationrights, op, value)
            }
            FieldRef::IsNeuromorphicHandoff => cmp_bool(ctx.is_neuromorphic_handoff, op, value),
        }
    }

    fn eval_flag(&self, flag: &FlagRef, ctx: &EvalContext) -> bool {
        match flag {
            FlagRef::NeurorightsForbidDecisionUse => ctx.neurorights.forbid_decision_use,
            FlagRef::NeurorightsSoulNonTradeable => ctx.neurorights.soul_non_tradeable,
            FlagRef::NeurorightsDreamstateSensitive => ctx.neurorights.dreamstate_sensitive,
        }
    }

    fn decision_from_expr(&self, expr: &DecisionExpr) -> RuleDecision {
        match expr {
            DecisionExpr::Allow => RuleDecision::Allow,
            DecisionExpr::Deny { reason } => RuleDecision::Deny { reason: reason.clone() },
            DecisionExpr::AllowWithConstraints { redactions, rate_limit } => {
                RuleDecision::AllowWithConstraints {
                    redactions: redactions.clone(),
                    notes: vec![format!("rate_limit={}", rate_limit)],
                }
            }
        }
    }
}

impl RuleEngine for DefaultRuleEngine {
    fn evaluate(&self, ctx: &EvalContext) -> RuleDecision {
        // Global RoH ceiling guard (hard invariant)
        if ctx.roh_after_estimate > self.roh_ceiling {
            return RuleDecision::Deny {
                reason: format!(
                    "RoH guard: roh_after {} exceeds ceiling {}",
                    ctx.roh_after_estimate, self.roh_ceiling
                ),
            };
        }

        for rule in self.sorted_rules() {
            if rule.scope != ctx.scope {
                continue;
            }
            if ctx.roh_after_estimate > rule.roh_guard.max_roh_after {
                // rule is not allowed to open more RoH than its slice
                continue;
            }
            if !self.eval_condition(&rule.when, ctx) {
                continue;
            }
            if rule.governed_by != ctx.token_class {
                continue;
            }
            return self.decision_from_expr(&rule.then);
        }

        RuleDecision::Deny {
            reason: "No matching rule; default deny.".into(),
        }
    }
}

/// ---------- Comparators ----------

fn cmp_string(actual: &str, op: &CmpOp, value: &Value) -> bool {
    match (op, value) {
        (CmpOp::Eq, Value::String(s)) => actual == s,
        (CmpOp::Ne, Value::String(s)) => actual != s,
        (CmpOp::InSet, Value::StringSet(set)) => set.iter().any(|v| v == actual),
        _ => false,
    }
}

fn cmp_f32(actual: f32, op: &CmpOp, value: &Value) -> bool {
    let target = match value {
        Value::F32(v) => *v,
        _ => return false,
    };
    match op {
        CmpOp::Eq => (actual - target).abs() < f32::EPSILON,
        CmpOp::Ne => (actual - target).abs() >= f32::EPSILON,
        CmpOp::Lt => actual < target,
        CmpOp::Le => actual <= target,
        CmpOp::Gt => actual > target,
        CmpOp::Ge => actual >= target,
        _ => false,
    }
}

fn cmp_bool(actual: bool, op: &CmpOp, value: &Value) -> bool {
    let target = match value {
        Value::Bool(v) => *v,
        _ => return false,
    };
    match op {
        CmpOp::Eq => actual == target,
        CmpOp::Ne => actual != target,
        _ => false,
    }
}

/// ---------- Example tests wiring BiophysicalUpload + NeuromorphicHandoff ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ctx(scope: RuleScope) -> EvalContext {
        EvalContext {
            route: "BiophysicalUpload".into(),
            token_class: TokenClass::EVOLVE,
            subject_id: "bostrom-subject-1".into(),
            roh_after_estimate: 0.2,
            lifeforce_cost: 0.1,
            neurorights: NeurorightsEnvelope {
                forbid_decision_use: true,
                soul_non_tradeable: true,
                dreamstate_sensitive: true,
            },
            capability: CapabilityScope {
                actuationrights: "LocalHostOnly".into(),
                biophysical_scope: "OfflineOnly".into(),
            },
            is_neuromorphic_handoff: false,
            scope,
        }
    }

    #[test]
    fn allow_biophysical_upload_with_constraints() {
        let json = include_str!("../rules/rulelogic.example.json");
        let engine = DefaultRuleEngine::from_json_str(json, 0.3).unwrap();

        let mut ctx = base_ctx(RuleScope::BiophysicalUpload);
        ctx.route = "BiophysicalUpload".into();

        let decision = engine.evaluate(&ctx);
        match decision {
            RuleDecision::AllowWithConstraints { redactions, notes } => {
                assert!(redactions.contains(&"raw_neuroaln".to_string()));
                assert!(notes.iter().any(|n| n.contains("rate_limit")));
            }
            other => panic!("expected AllowWithConstraints, got {:?}", other),
        }
    }

    #[test]
    fn allow_offline_neuromorphic_handoff() {
        let json = include_str!("../rules/rulelogic.example.json");
        let engine = DefaultRuleEngine::from_json_str(json, 0.3).unwrap();

        let mut ctx = base_ctx(RuleScope::NeuromorphicHandoff);
        ctx.scope = RuleScope::NeuromorphicHandoff;
        ctx.route = "NeuromorphicHandoff".into();
        ctx.is_neuromorphic_handoff = true;

        let decision = engine.evaluate(&ctx);
        match decision {
            RuleDecision::AllowWithConstraints { notes, .. } => {
                assert!(notes.iter().any(|n| n.contains("offline-only")));
            }
            other => panic!("expected AllowWithConstraints for neuromorphic handoff, got {:?}", other),
        }
    }

    #[test]
    fn deny_roh_over_ceiling() {
        let json = include_str!("../rules/rulelogic.example.json");
        let engine = DefaultRuleEngine::from_json_str(json, 0.3).unwrap();

        let mut ctx = base_ctx(RuleScope::BiophysicalUpload);
        ctx.roh_after_estimate = 0.5;

        let decision = engine.evaluate(&ctx);
        match decision {
            RuleDecision::Deny { reason } => {
                assert!(reason.contains("RoH guard"));
            }
            other => panic!("expected RoH guard deny, got {:?}", other),
        }
    }
}
