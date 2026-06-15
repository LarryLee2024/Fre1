//! 治疗执行器 — HealExecution
//!
//! 计算治疗量：
//! - 基础治疗：HealPower（来自 source_attrs 或 base_value）
//! - 治疗加成：HealPower * (1 + HealBonus)

use super::{Execution, ExecutionContext, ExecutionResult, StepRecord};

/// 治疗执行器
pub struct HealExecution;

impl Execution for HealExecution {
    fn type_name(&self) -> &'static str {
        "Heal"
    }

    fn calculate(&self, ctx: &ExecutionContext) -> ExecutionResult {
        let mut breakdown = Vec::new();

        // 获取基础治疗值（来自 base_value 或 source_attrs.attack 作为治疗力）
        let base_heal = if ctx.base_value > 0.0 {
            ctx.base_value
        } else {
            ctx.source_attrs.attack
        };

        // 获取治疗加成
        let heal_bonus = ctx
            .execution_params
            .get("heal_bonus")
            .copied()
            .unwrap_or(0.0);

        // Step 1: 基础治疗
        breakdown.push(StepRecord {
            name: "base_heal".to_string(),
            input: base_heal,
            output: base_heal,
        });

        // Step 2: 治疗加成
        let after_bonus = base_heal * (1.0 + heal_bonus);
        breakdown.push(StepRecord {
            name: "heal_bonus".to_string(),
            input: base_heal,
            output: after_bonus,
        });

        // Step 3: 最终治疗量
        breakdown.push(StepRecord {
            name: "final_heal".to_string(),
            input: after_bonus,
            output: after_bonus,
        });

        ExecutionResult {
            value: after_bonus as i32,
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
    fn heal_calculation_basic() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 50.0, // 作为治疗力
                defense: 0.0,
                ..default()
            },
            target_attrs: super::super::AttributeSnapshot::default(),
            base_value: 0.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: HashMap::new(),
            terrain_id: None,
            is_skill: false,
        };

        let executor = HealExecution;
        let result = executor.calculate(&ctx);

        assert_eq!(result.value, 50);
        assert!(!result.is_critical);
    }

    #[test]
    fn heal_calculation_with_bonus() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 100.0,
                defense: 0.0,
                ..default()
            },
            target_attrs: super::super::AttributeSnapshot::default(),
            base_value: 0.0,
            modifier_value: 0,
            stack_count: 1,
            execution_params: {
                let mut params = HashMap::new();
                params.insert("heal_bonus".to_string(), 0.5);
                params
            },
            terrain_id: None,
            is_skill: false,
        };

        let executor = HealExecution;
        let result = executor.calculate(&ctx);

        // 100 * (1 + 0.5) = 150
        assert_eq!(result.value, 150);
    }
}
