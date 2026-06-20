//! Rule Engine — 统一规则评估引擎
//!
//! 纯函数规则评估，无副作用。
//! 递归遍历规则列表，对每条规则评估其条件，条件满足时输出效果。
//!
//! 详见 docs/02-domain/capabilities/rule_domain.md。

use bevy::prelude::*;

use crate::core::capabilities::condition::foundation::{ConditionContext, ConditionResult};
use crate::core::capabilities::condition::mechanism::evaluator::evaluate;
use crate::core::capabilities::rule::foundation::{RuleDef, RuleEffect};

/// 规则评估结果——包含规则 ID 和匹配的效果。
#[derive(Debug, Clone)]
pub struct RuleMatch {
    /// 匹配的规则 ID
    pub rule_id: String,
    /// 规则效果
    pub effect: RuleEffect,
    /// 规则优先级
    pub priority: u32,
}

/// 评估一组规则，返回所有条件满足的效果。
///
/// 纯函数：不修改任何 ECS 状态，仅读取条件上下文。
/// 按优先级排序输出（priority 越小越优先）。
///
/// # Arguments
/// * `rules` — 待评估的规则列表
/// * `context` — 条件评估上下文（实体标签、属性等）
/// * `entity` — 被评估的实体
/// * `commands` — 用于触发条件事件
///
/// # Returns
/// 所有条件满足的规则效果，按优先级排序。
pub fn evaluate_rules(
    rules: &[RuleDef],
    context: &ConditionContext,
    entity: Entity,
    commands: &mut Commands,
) -> Vec<RuleMatch> {
    let mut matches: Vec<RuleMatch> = rules
        .iter()
        .filter(|rule| rule.enabled)
        .filter(|rule| {
            evaluate(&rule.condition, context, entity, commands).is_passed()
        })
        .map(|rule| RuleMatch {
            rule_id: rule.id.clone(),
            effect: rule.effect.clone(),
            priority: rule.priority,
        })
        .collect();

    // 按优先级排序（越小越优先）
    matches.sort_by_key(|m| m.priority);
    matches
}

/// 评估单条规则，返回是否匹配。
///
/// 纯函数：不修改任何 ECS 状态。
pub fn evaluate_single_rule(
    rule: &RuleDef,
    context: &ConditionContext,
    entity: Entity,
    commands: &mut Commands,
) -> ConditionResult {
    if !rule.enabled {
        return ConditionResult::failed("rule is disabled");
    }
    evaluate(&rule.condition, context, entity, commands)
}
