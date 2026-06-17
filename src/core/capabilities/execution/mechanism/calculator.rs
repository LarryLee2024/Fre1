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
