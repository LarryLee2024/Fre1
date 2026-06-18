//! Condition Integration — combat 域接入 condition capability
//!
//! 封装 condition capability 的免疫检查和条件评估，
//! 用于效果应用前的前置校验。

use crate::core::capabilities::condition::foundation::{
    Condition, ConditionContext, ConditionResult,
};
use crate::core::capabilities::condition::mechanism::evaluator::{check_immunity, evaluate};

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗条件 Facade — 封装 condition capability 的战斗相关检查。
pub struct CombatConditionFacade;

impl CombatConditionFacade {
    /// 检查目标是否对指定效果类型免疫。
    ///
    /// 免疫条件具有最高优先级（不变量 §3.5）。
    pub fn check_effect_immunity(context: &ConditionContext, effect_type: &str) -> ConditionResult {
        check_immunity(context, effect_type)
    }

    /// 评估一个条件是否满足。
    pub fn evaluate_condition(
        condition: &Condition,
        context: &ConditionContext,
    ) -> ConditionResult {
        evaluate(condition, context)
    }

    /// 检查施法者是否满足施法条件（沉默、束缚等）。
    ///
    /// 常用条件组合：
    /// - Not(TagRequirement("Silenced")) — 未被沉默
    /// - Not(TagRequirement("Paralyzed")) — 未被束缚
    pub fn check_casting_conditions(
        context: &ConditionContext,
        additional: &[Condition],
    ) -> ConditionResult {
        // 基础检查：未被沉默
        let silence_check = Condition::TagRequirement {
            mode: crate::core::capabilities::condition::foundation::TagRequirementMode::Not,
            tag_id: "Silenced".to_string(),
        };
        let result = evaluate(&silence_check, context);
        if !result.is_passed() {
            return result;
        }

        // 基础检查：未被束缚
        let paralyze_check = Condition::TagRequirement {
            mode: crate::core::capabilities::condition::foundation::TagRequirementMode::Not,
            tag_id: "Paralyzed".to_string(),
        };
        let result = evaluate(&paralyze_check, context);
        if !result.is_passed() {
            return result;
        }

        // 额外条件检查
        for condition in additional {
            let result = evaluate(condition, context);
            if !result.is_passed() {
                return result;
            }
        }

        ConditionResult::passed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn immunity_check_passed_when_no_immune_tag() {
        let context = ConditionContext {
            tag_ids: Some(vec!["Fire".to_string()]),
            attribute_values: HashMap::new(),
            resource_values: HashMap::new(),
        };
        let result = CombatConditionFacade::check_effect_immunity(&context, "Fire");
        assert!(result.is_passed());
    }

    #[test]
    fn immunity_check_failed_when_immune_tag_present() {
        let context = ConditionContext {
            tag_ids: Some(vec!["Immune.Fire".to_string()]),
            attribute_values: HashMap::new(),
            resource_values: HashMap::new(),
        };
        let result = CombatConditionFacade::check_effect_immunity(&context, "Fire");
        assert!(!result.is_passed());
    }
}
