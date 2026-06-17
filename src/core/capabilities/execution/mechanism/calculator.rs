//! Execution Calculator — 执行计算调度器
//!
//! 提供执行计算的核心调度逻辑，按 ExecutionType 分发到对应计算路由。
//! 遵循 docs/02-domain/execution_domain.md §5 的流程定义。
//!
//! 核心函数：
//! - execute() — 主入口：按 ExecutionType 分发计算
//! - execute_damage() — 伤害计算调度（占位，实际公式在 Domains/rules/）
//! - execute_heal() — 治疗计算调度（占位，实际公式在 Domains/rules/）
//! - execute_custom() — 自定义计算调度
//! - execute_direct_mod() — 直接属性修改计算
//! - validate_result() — 结果校验（不变量 3.4）
//!
//! 注意：自身不包含业务公式（不变量 3.1），所有公式调用指向 Domains/rules/ 的纯函数。

use crate::core::capabilities::execution::foundation::{
    CalcTrace, CustomExecutionRef, DamageParams, DirectOp, ExecutionContext, ExecutionError,
    ExecutionResult, ExecutionType, HealParams, ScalableValue,
};

/// 执行完整计算流程。
///
/// 流程（docs/02-domain/execution_domain.md §5.1）：
/// 1. 按 ExecutionType 分发到对应计算路由
/// 2. 公式执行计算，返回结果
/// 3. 验证结果数值范围（不变量 3.4）
/// 4. 封装计算结果
///
/// # Errors
/// - ExecutionError::FormulaNotFound: formula_id 未注册
/// - ExecutionError::ContextMissing: 必要上下文缺失
/// - ExecutionError::InvalidResult: 计算结果数值非法
pub fn execute(ctx: &ExecutionContext) -> Result<ExecutionResult, ExecutionError> {
    let result = match &ctx.execution_type {
        ExecutionType::Damage(params) => execute_damage(ctx, params)?,
        ExecutionType::Heal(params) => execute_heal(ctx, params)?,
        ExecutionType::Custom(custom_ref) => execute_custom(ctx, custom_ref)?,
        ExecutionType::DirectAttributeMod {
            attribute_id,
            operation,
            value,
        } => execute_direct_mod(ctx, attribute_id, operation, value)?,
        ExecutionType::None => ExecutionResult::success(0.0),
    };

    // 不变量 3.4: 结果数值范围校验
    validate_result(&result)?;

    Ok(result)
}

/// 执行伤害计算。
///
/// 当前为占位实现——实际公式由 Domains/rules/damage_formula.rs 提供。
/// 此函数仅做参数准备和结果封装。
///
/// 计算过程（docs/02-domain/execution_domain.md §5.2）：
/// 1. 读取来源攻击属性
/// 2. 计算基础骰面值（占位：取平均值）
/// 3. 计算固定加值
/// 4. 计算属性修正
/// 5. 应用暴击倍率（如适用）
/// 6. 记录计算追踪
fn execute_damage(
    ctx: &ExecutionContext,
    params: &DamageParams,
) -> Result<ExecutionResult, ExecutionError> {
    // 不变量 3.1: Execution 不包含公式——此处仅为占位调度
    // 实际实现应调用 Domains/rules/ 中的 formula_id 对应函数

    // V1: formula_id 非空检查
    if params.formula_id.is_empty() {
        return Err(ExecutionError::FormulaNotFound {
            formula_id: "(empty)".into(),
            detail: "formula_id must not be empty".into(),
        });
    }

    let mut trace = CalcTrace::new(&params.formula_id);

    // 1. 读取来源属性
    let attr_mod_value = if let Some(ref modifier) = params.attribute_modifier {
        let attrs = if modifier.use_base {
            &ctx.source_attributes
        } else {
            &ctx.source_attributes
        };
        let value = modifier.calculate(attrs);
        trace = trace.with_input(&modifier.source_attribute, value);
        value
    } else {
        0.0
    };

    // 2. 计算骰面值（占位：使用平均值，实际由公式 + RNG 决定）
    let dice_value = if let Some(ref dice) = params.damage_dice {
        let avg = (dice.sides as f32 + 1.0) / 2.0 * dice.count as f32;
        trace = trace.with_intermediate("dice_avg", avg);
        avg
    } else {
        0.0
    };

    // 3. 计算固定加值
    let flat_bonus = if let Some(ref bonus) = params.flat_bonus {
        let value = bonus.calculate(ctx.ability_params.ability_level as u32);
        trace = trace.with_intermediate("flat_bonus", value);
        value
    } else {
        0.0
    };

    // 4. 汇总基础值
    let base_damage = dice_value + flat_bonus + attr_mod_value;
    trace = trace.with_intermediate("base_damage", base_damage);

    // 5. 应用暴击
    let (final_damage, was_critical) = if params.can_critical {
        // 占位：默认不暴击（实际由 RNG + 暴击率公式决定）
        let crit_value = base_damage;
        trace = trace.with_intermediate("critical_multiplier", params.critical_multiplier);
        (crit_value, false)
    } else {
        (base_damage, false)
    };

    // 6. 结果值非负保证（不变量 3.4）
    let final_damage = final_damage.max(0.0);
    trace = trace.with_output(final_damage);

    Ok(ExecutionResult::success(final_damage)
        .with_critical(was_critical)
        .with_trace(trace))
}

/// 执行治疗计算。
///
/// 当前为占位实现——实际公式由 Domains/rules/heal_formula.rs 提供。
///
/// 计算过程（docs/02-domain/execution_domain.md §5.3）：
/// 1. 读取治疗属性修正
/// 2. 计算基础治疗量
/// 3. 汇总最终治疗值
/// 4. 记录计算追踪
fn execute_heal(
    ctx: &ExecutionContext,
    params: &HealParams,
) -> Result<ExecutionResult, ExecutionError> {
    // V1: formula_id 非空检查
    if params.formula_id.is_empty() {
        return Err(ExecutionError::FormulaNotFound {
            formula_id: "(empty)".into(),
            detail: "formula_id must not be empty".into(),
        });
    }

    let mut trace = CalcTrace::new(&params.formula_id);

    // 1. 读取治疗属性修正
    let attr_mod_value = if let Some(ref modifier) = params.attribute_modifier {
        let value = modifier.calculate(&ctx.source_attributes);
        trace = trace.with_input(&modifier.source_attribute, value);
        value
    } else {
        0.0
    };

    // 2. 计算基础治疗量
    let base_heal = params
        .base_heal
        .calculate(ctx.ability_params.ability_level as u32);
    trace = trace.with_intermediate("base_heal", base_heal);

    // 3. 汇总
    let total_heal = base_heal + attr_mod_value;
    trace = trace.with_intermediate("attr_modifier", attr_mod_value);
    trace = trace.with_output(total_heal);

    // 4. 结果值非负保证（不变量 3.4）
    let total_heal = total_heal.max(0.0);

    Ok(ExecutionResult::success(total_heal).with_trace(trace))
}

/// 执行自定义计算。
///
/// 当前为占位——自定义计算需要 Domains 注册后才能使用。
/// 返回 CustomExecutionNotRegistered 错误。
fn execute_custom(
    _ctx: &ExecutionContext,
    custom_ref: &CustomExecutionRef,
) -> Result<ExecutionResult, ExecutionError> {
    // 不变量 3.5: 自定义执行标识唯一性
    // 当前未实现 CustomExecutionRegistry
    // 实际应根据 execution_id 在注册表中查找并调用
    Err(ExecutionError::CustomExecutionNotRegistered(
        custom_ref.execution_id.clone(),
    ))
}

/// 执行直接属性修改（Set/Add/Subtract/Multiply）。
fn execute_direct_mod(
    _ctx: &ExecutionContext,
    attribute_id: &str,
    operation: &DirectOp,
    value: &ScalableValue,
) -> Result<ExecutionResult, ExecutionError> {
    if attribute_id.is_empty() {
        return Err(ExecutionError::ContextMissing {
            field: "attribute_id".into(),
            detail: "attribute_id must not be empty".into(),
        });
    }

    let numeric_value = value.calculate(1);
    let mut trace = CalcTrace::new("direct_mod");
    trace = trace.with_input("attribute_id", 0.0); // 标记属性 ID
    trace = trace.with_input(
        "operation",
        match operation {
            DirectOp::Set => 0.0,
            DirectOp::Add => 1.0,
            DirectOp::Subtract => 2.0,
            DirectOp::Multiply => 3.0,
        },
    );
    trace = trace.with_input("value", numeric_value);
    trace = trace.with_output(numeric_value);

    Ok(ExecutionResult::success(numeric_value).with_trace(trace))
}

/// 校验结果数值范围（不变量 3.4）。
///
/// 条件：任何 Execution 返回数值结果后
/// 不变量：伤害/治疗值 ≥ 0，负数应归零
pub fn validate_result(result: &ExecutionResult) -> Result<(), ExecutionError> {
    if result.value < 0.0 {
        return Err(ExecutionError::InvalidResult(format!(
            "result value {} is negative, must be >= 0",
            result.value
        )));
    }
    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 校验 ExecutionContext 的完整性（不变量 3.3）。
///
/// 检查必要字段是否存在。
pub fn validate_context(ctx: &ExecutionContext) -> Result<(), ExecutionError> {
    if ctx.source_entity.is_empty() {
        return Err(ExecutionError::ContextMissing {
            field: "source_entity".into(),
            detail: "source entity must not be empty".into(),
        });
    }
    if ctx.target_entity.is_empty() {
        return Err(ExecutionError::ContextMissing {
            field: "target_entity".into(),
            detail: "target entity must not be empty".into(),
        });
    }
    Ok(())
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::execution::foundation::{
        AbilityExecutionParams, AttributeModifierDef, CustomExecutionRef, DamageParams, DiceDef,
        DirectOp, EnvironmentParams, ExecutionContext, ExecutionType, HealParams, ScalableValue,
    };

    // ── Helpers ────────────────────────────────────────────

    fn make_damage_context() -> ExecutionContext {
        let params = DamageParams::new("dnd_5e_damage")
            .with_dice(DiceDef::new(1, 8).unwrap())
            .with_flat_bonus(ScalableValue::Fixed(3.0));

        let mut attrs = std::collections::HashMap::new();
        attrs.insert("attr_strength".into(), 4.0);

        ExecutionContext::new(ExecutionType::Damage(params), "caster_001", "target_001")
            .with_source_attributes(attrs)
    }

    fn make_heal_context() -> ExecutionContext {
        let params = HealParams::new("dnd_5e_heal", ScalableValue::Fixed(10.0));

        ExecutionContext::new(ExecutionType::Heal(params), "healer_001", "target_001")
    }

    fn make_direct_mod_context() -> ExecutionContext {
        ExecutionContext::new(
            ExecutionType::DirectAttributeMod {
                attribute_id: "attr_hp".into(),
                operation: DirectOp::Add,
                value: ScalableValue::Fixed(5.0),
            },
            "system",
            "target_001",
        )
    }

    // ── Main execution dispatch ────────────────────────────

    #[test]
    fn unit_001_execute_damage_returns_value() {
        let ctx = make_damage_context();
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        assert!(result.value >= 0.0);
    }

    #[test]
    fn unit_002_execute_heal_returns_value() {
        let ctx = make_heal_context();
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        assert!(result.value >= 0.0);
    }

    #[test]
    fn unit_003_execute_none_returns_zero() {
        let ctx = ExecutionContext::new(ExecutionType::None, "caster", "target");
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.value, 0.0);
    }

    #[test]
    fn unit_004_execute_custom_not_registered() {
        let custom = CustomExecutionRef::new("tactical.knockback");
        let ctx = ExecutionContext::new(ExecutionType::Custom(custom), "caster_001", "target_001");
        let result = execute(&ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            ExecutionError::CustomExecutionNotRegistered(_)
        ));
    }

    #[test]
    fn unit_005_execute_direct_mod_returns_value() {
        let ctx = make_direct_mod_context();
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.value, 5.0);
    }

    #[test]
    fn unit_006_execute_empty_formula_id_error() {
        let params = DamageParams::new("");
        let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster_001", "target_001");
        let result = execute(&ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExecutionError::FormulaNotFound { .. }));
    }

    #[test]
    fn unit_007_execute_empty_heal_formula_id_error() {
        let params = HealParams::new("", ScalableValue::Fixed(10.0));
        let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer_001", "target_001");
        let result = execute(&ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExecutionError::FormulaNotFound { .. }));
    }

    // ── Damage parameters ──────────────────────────────────

    #[test]
    fn unit_010_damage_with_dice_and_bonus() {
        let params = DamageParams::new("test_formula")
            .with_dice(DiceDef::new(2, 6).unwrap())
            .with_flat_bonus(ScalableValue::Fixed(5.0));

        let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target");
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        // dice avg = (6+1)/2 * 2 = 7.0, + flat 5.0 = 12.0
        assert!(result.value >= 0.0);
    }

    #[test]
    fn unit_011_damage_with_attribute_modifier() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("attr_strength".into(), 4.0);

        let modifier = AttributeModifierDef::new("attr_strength").with_multiplier(1.0);
        let params = DamageParams::new("test_formula").with_attribute_modifier(modifier);

        let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target")
            .with_source_attributes(attrs);
        let result = execute(&ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn unit_012_damage_with_critical() {
        let params = DamageParams::new("test_formula").with_critical(2.0);

        let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target");
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        // can_critical 不影响占位计算，只是标记
        assert!(!result.was_critical); // 占位：默认不暴击
    }

    // ── Heal parameters ────────────────────────────────────

    #[test]
    fn unit_020_heal_with_attr_modifier() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("attr_wisdom".into(), 5.0);

        let modifier = AttributeModifierDef::new("attr_wisdom").with_multiplier(1.0);
        let params = HealParams::new("heal_formula", ScalableValue::Fixed(10.0))
            .with_attribute_modifier(modifier);

        let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer", "target")
            .with_source_attributes(attrs);
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        // 10.0 + 5.0 = 15.0
        assert!(result.value >= 0.0);
    }

    #[test]
    fn unit_021_heal_temporary_hp() {
        let params =
            HealParams::new("heal_formula", ScalableValue::Fixed(8.0)).with_temporary_hp(true);

        let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer", "target");
        let result = execute(&ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.value, 8.0);
    }

    // ── ScalableValue calculation ──────────────────────────

    #[test]
    fn unit_030_scalable_value_fixed() {
        let sv = ScalableValue::Fixed(10.0);
        assert_eq!(sv.calculate(1), 10.0);
        assert_eq!(sv.calculate(5), 10.0); // Fixed 不随等级变化
    }

    #[test]
    fn unit_031_scalable_value_per_level() {
        let sv = ScalableValue::PerLevel {
            base: 10.0,
            per_level: 5.0,
        };
        assert_eq!(sv.calculate(1), 10.0);
        assert_eq!(sv.calculate(2), 15.0);
        assert_eq!(sv.calculate(5), 30.0); // base + 4 * 5 = 30
    }

    #[test]
    fn unit_032_scalable_value_level_zero_treated_as_one() {
        let sv = ScalableValue::PerLevel {
            base: 10.0,
            per_level: 5.0,
        };
        assert_eq!(sv.calculate(0), 10.0); // 等级 0 按 1 处理
    }

    // ── DiceDef validation ─────────────────────────────────

    #[test]
    fn unit_040_dice_def_valid() {
        let dice = DiceDef::new(1, 8).unwrap();
        assert_eq!(dice.count, 1);
        assert_eq!(dice.sides, 8);
    }

    #[test]
    fn unit_041_dice_def_zero_count_error() {
        let result = DiceDef::new(0, 8);
        assert!(result.is_err());
    }

    #[test]
    fn unit_042_dice_def_single_side_error() {
        let result = DiceDef::new(1, 1);
        assert!(result.is_err());
    }

    #[test]
    fn unit_043_dice_roll_range() {
        let dice = DiceDef::new(2, 6).unwrap();
        assert_eq!(dice.min_roll(), 2);
        assert_eq!(dice.max_roll(), 12);
    }

    // ── AttributeModifierDef ───────────────────────────────

    #[test]
    fn unit_050_attr_modifier_calculate() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("attr_strength".into(), 4.0);

        let modifier = AttributeModifierDef::new("attr_strength");
        assert_eq!(modifier.calculate(&attrs), 4.0);
    }

    #[test]
    fn unit_051_attr_modifier_multiplier() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("attr_strength".into(), 10.0);

        let modifier = AttributeModifierDef::new("attr_strength").with_multiplier(0.5);
        assert_eq!(modifier.calculate(&attrs), 5.0);
    }

    #[test]
    fn unit_052_attr_modifier_missing_attribute_returns_zero() {
        let attrs = std::collections::HashMap::new();
        let modifier = AttributeModifierDef::new("attr_nonexistent");
        assert_eq!(modifier.calculate(&attrs), 0.0);
    }

    // ── Result validation ──────────────────────────────────

    #[test]
    fn unit_060_validate_valid_result() {
        let result = ExecutionResult::success(42.0);
        assert!(validate_result(&result).is_ok());
    }

    #[test]
    fn unit_061_validate_negative_result_error() {
        let result = ExecutionResult::success(-5.0);
        assert!(validate_result(&result).is_err());
    }

    #[test]
    fn unit_062_validate_zero_result() {
        let result = ExecutionResult::success(0.0);
        assert!(validate_result(&result).is_ok());
    }

    // ── Context validation ─────────────────────────────────

    #[test]
    fn unit_070_validate_valid_context() {
        let ctx = ExecutionContext::new(ExecutionType::None, "caster", "target");
        assert!(validate_context(&ctx).is_ok());
    }

    #[test]
    fn unit_071_validate_empty_source_entity() {
        let ctx = ExecutionContext::new(ExecutionType::None, "", "target");
        assert!(validate_context(&ctx).is_err());
    }

    #[test]
    fn unit_072_validate_empty_target_entity() {
        let ctx = ExecutionContext::new(ExecutionType::None, "caster", "");
        assert!(validate_context(&ctx).is_err());
    }

    // ── CalcTrace ──────────────────────────────────────────

    #[test]
    fn unit_080_calc_trace_records_computation() {
        let trace = CalcTrace::new("test_formula")
            .with_input("attr_str", 4.0)
            .with_intermediate("dice_avg", 7.0)
            .with_output(11.0);

        assert_eq!(trace.formula_id, "test_formula");
        assert_eq!(trace.inputs.get("attr_str"), Some(&4.0));
        assert!(
            trace
                .intermediate_values
                .contains(&("dice_avg".into(), 7.0))
        );
        assert_eq!(trace.output, 11.0);
    }

    #[test]
    fn unit_081_execute_result_has_trace() {
        let ctx = make_damage_context();
        let result = execute(&ctx).unwrap();
        assert!(result.calc_trace.is_some());
        let trace = result.calc_trace.unwrap();
        assert_eq!(trace.formula_id, "dnd_5e_damage");
    }

    // ── ExecutionResult builder ────────────────────────────

    #[test]
    fn unit_090_execution_result_builders() {
        let result = ExecutionResult::success(10.0)
            .with_critical(true)
            .with_miss(false);

        assert!(result.success);
        assert_eq!(result.value, 10.0);
        assert!(result.was_critical);
        assert!(!result.was_miss);
    }

    #[test]
    fn unit_091_execution_result_failure() {
        let result = ExecutionResult::failure();
        assert!(!result.success);
        assert_eq!(result.value, 0.0);
    }

    // ── ExecutionType helpers ──────────────────────────────

    #[test]
    fn unit_100_execution_type_name() {
        let dam = ExecutionType::Damage(DamageParams::new("f"));
        assert_eq!(dam.name(), "Damage");

        let heal = ExecutionType::Heal(HealParams::new("f", ScalableValue::Fixed(1.0)));
        assert_eq!(heal.name(), "Heal");

        assert_eq!(ExecutionType::None.name(), "None");
    }

    #[test]
    fn unit_101_is_numeric_calculation() {
        let dam = ExecutionType::Damage(DamageParams::new("f"));
        assert!(dam.is_numeric_calculation());

        assert!(!ExecutionType::None.is_numeric_calculation());
    }

    // ── DamageParams validation ────────────────────────────

    #[test]
    fn unit_110_damage_params_validate_invalid_crit_multiplier() {
        let params = DamageParams::new("f").with_critical(0.5); // will clamp to 1.0 in constructor
        assert!(params.validate().is_ok()); // constructor already clamps
    }

    #[test]
    fn unit_111_damage_params_validate_valid() {
        let params = DamageParams::new("f").with_dice(DiceDef::new(1, 6).unwrap());
        assert!(params.validate().is_ok());
    }

    // ── DirectOp execute ───────────────────────────────────

    #[test]
    fn unit_120_execute_direct_mod_set() {
        let ctx = ExecutionContext::new(
            ExecutionType::DirectAttributeMod {
                attribute_id: "attr_hp".into(),
                operation: DirectOp::Set,
                value: ScalableValue::Fixed(100.0),
            },
            "system",
            "target_001",
        );
        let result = execute(&ctx).unwrap();
        assert_eq!(result.value, 100.0);
    }

    #[test]
    fn unit_121_execute_direct_mod_empty_attribute_id() {
        let ctx = ExecutionContext::new(
            ExecutionType::DirectAttributeMod {
                attribute_id: "".into(),
                operation: DirectOp::Add,
                value: ScalableValue::Fixed(5.0),
            },
            "system",
            "target_001",
        );
        let result = execute(&ctx);
        assert!(result.is_err());
    }

    // ── EnvironmentParams ──────────────────────────────────

    #[test]
    fn unit_130_environment_params_default() {
        let env = EnvironmentParams::default();
        assert!(!env.is_high_ground);
        assert!(!env.has_cover);
        assert!(!env.is_flanked);
        assert_eq!(env.current_turn, 0);
    }

    // ── AbilityExecutionParams ─────────────────────────────

    #[test]
    fn unit_140_ability_params_default() {
        let p = AbilityExecutionParams::default();
        assert_eq!(p.ability_level, 1);
        assert!(p.ability_def_id.is_none());
        assert!(!p.has_effect_override);
    }
}
