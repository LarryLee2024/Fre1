//! 护盾执行器 — ShieldExecution
//!
//! 计算护盾吸收量：
//! - 基础护盾：ShieldPower
//! - 护盾加成：ShieldPower * (1 + ShieldBonus)

use super::{Execution, ExecutionContext, ExecutionResult, StepRecord};

/// 护盾执行器
pub struct ShieldExecution;

impl Execution for ShieldExecution {
    fn type_name(&self) -> &'static str {
        "Shield"
    }

    fn calculate(&self, ctx: &ExecutionContext) -> ExecutionResult {
        let mut breakdown = Vec::new();

        // 获取基础护盾值（来自 base_value 或 source_attrs.attack 作为护盾力）
        let base_shield = if ctx.base_value > 0.0 {
            ctx.base_value
        } else {
            ctx.source_attrs.attack
        };

        // 获取护盾加成
        let shield_bonus = ctx
            .execution_params
            .get("shield_bonus")
            .copied()
            .unwrap_or(0.0);

        // Step 1: 基础护盾
        breakdown.push(StepRecord {
            name: "base_shield".to_string(),
            input: base_shield,
            output: base_shield,
        });

        // Step 2: 护盾加成
        let after_bonus = base_shield * (1.0 + shield_bonus);
        breakdown.push(StepRecord {
            name: "shield_bonus".to_string(),
            input: base_shield,
            output: after_bonus,
        });

        // Step 3: 最终护盾值
        breakdown.push(StepRecord {
            name: "final_shield".to_string(),
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
    fn shield_calculation_basic() {
        let ctx = ExecutionContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs: super::super::AttributeSnapshot {
                attack: 80.0, // 作为护盾力
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

        let executor = ShieldExecution;
        let result = executor.calculate(&ctx);

        assert_eq!(result.value, 80);
        assert!(!result.is_critical);
    }

    #[test]
    fn shield_calculation_with_bonus() {
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
                params.insert("shield_bonus".to_string(), 0.25);
                params
            },
            terrain_id: None,
            is_skill: false,
        };

        let executor = ShieldExecution;
        let result = executor.calculate(&ctx);

        // 100 * (1 + 0.25) = 125
        assert_eq!(result.value, 125);
    }
}
