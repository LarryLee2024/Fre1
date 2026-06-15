//! 伤害执行器 — DamageExecution
//!
//! 计算伤害值：
//! - 普通伤害：Attack - Defense
//! - 真实伤害：Attack（忽略防御）
//! - 暴击伤害：Attack * CritMultiplier

use super::{Execution, ExecutionContext, ExecutionResult, StepRecord};

/// 伤害执行器
pub struct DamageExecution;

impl Execution for DamageExecution {
    fn type_name(&self) -> &'static str {
        "Damage"
    }

    fn calculate(&self, ctx: &ExecutionContext) -> ExecutionResult {
        let mut breakdown = Vec::new();
        let mut value = 0.0;

        let effective_atk = ctx.source_attrs.attack;
        let effective_def = ctx.target_attrs.defense;
        let base_def = ctx.target_attrs.defense;

        // 获取参数
        let multiplier = ctx.base_value;
        let ignore_def_percent = ctx
            .execution_params
            .get("ignore_def_percent")
            .copied()
            .unwrap_or(0.0);

        // Step 1: 基础攻击力
        breakdown.push(StepRecord {
            name: "base_attack".to_string(),
            input: effective_atk,
            output: effective_atk,
        });
        value = effective_atk;

        // Step 2: 应用倍率
        let after_multiplier = effective_atk * multiplier;
        breakdown.push(StepRecord {
            name: "multiplier".to_string(),
            input: effective_atk,
            output: after_multiplier,
        });
        value = after_multiplier;

        // Step 3: 防御计算
        let effective_def_after_ignore = effective_def * (1.0 - ignore_def_percent);
        let defense_reduction = effective_def_after_ignore.min(value);
        let after_defense = (value - defense_reduction).max(1.0);

        breakdown.push(StepRecord {
            name: "defense_reduction".to_string(),
            input: effective_def_after_ignore,
            output: defense_reduction,
        });

        value = after_defense;

        // Step 4: 最终伤害（最低为1）
        breakdown.push(StepRecord {
            name: "final_damage".to_string(),
            input: value,
            output: value,
        });

        ExecutionResult {
            value: value as i32,
            breakdown,
            is_critical: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn 伤害计算_基础() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 100.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: super::super::AttributeSnapshot {
                attack: 0.0,
                defense: 20.0,
                ..default()
            },
            base_value: 1.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };

        let executor = DamageExecution;
        let result = executor.calculate(&ctx);

        // 100 * 1.0 - 20 = 80
        assert_eq!(result.value, 80);
        assert!(!result.is_critical);
        assert!(!result.breakdown.is_empty());
    }

    #[test]
    fn 伤害计算_带倍率() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 100.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: super::super::AttributeSnapshot {
                attack: 0.0,
                defense: 20.0,
                ..default()
            },
            base_value: 1.5,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: true,
        };

        let executor = DamageExecution;
        let result = executor.calculate(&ctx);

        // 100 * 1.5 - 20 = 130
        assert_eq!(result.value, 130);
    }

    #[test]
    fn 伤害计算_最低为1() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 10.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: super::super::AttributeSnapshot {
                attack: 0.0,
                defense: 100.0,
                ..default()
            },
            base_value: 1.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };

        let executor = DamageExecution;
        let result = executor.calculate(&ctx);

        // 最低伤害为1
        assert_eq!(result.value, 1);
    }
}
