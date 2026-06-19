//! Condition 评估器
//!
//! 纯函数条件评估，无副作用（领域规则 §3.1 不变量）。
//! 递归遍历 Condition 树，对叶子节点分发到对应检查逻辑。
//!
//! 详见 docs/02-domain/capabilities/condition_domain.md §5.1-5.3。

use crate::core::capabilities::condition::foundation::{
    Condition, ConditionContext, ConditionResult, TagRequirementMode,
};

/// 评估单个条件。
///
/// 递归处理条件树：
/// - 叶子节点（TagRequirement / AttributeCheck / ResourceCheck）→ 直接评估
/// - 组合节点（And / Or / Not）→ 递归评估后组合
///
/// # 不变量保证
/// - 无副作用（§3.1）：纯函数，不修改任何外部状态
/// - 确定性（§3.3）：同一输入始终返回同一结果
/// - 引用存在性（§3.2）：标签/属性不存在时返回 Failed
pub fn evaluate(condition: &Condition, context: &ConditionContext) -> ConditionResult {
    match condition {
        Condition::TagRequirement { mode, tag_id } => {
            evaluate_tag_requirement(*mode, tag_id, context)
        }
        Condition::AttributeCheck {
            attribute_id,
            operator,
            threshold,
        } => evaluate_attribute_check(attribute_id, *operator, *threshold, context),
        Condition::ResourceCheck {
            resource_id,
            required_amount,
        } => evaluate_resource_check(resource_id, *required_amount, context),
        Condition::And(children) => evaluate_and(children, context),
        Condition::Or(children) => evaluate_or(children, context),
        Condition::Not(child) => {
            let result = evaluate(child, context);
            if result.is_passed() {
                ConditionResult::failed("NOT condition: child passed, negated to fail")
            } else {
                ConditionResult::passed()
            }
        }
        Condition::Custom(_custom) => {
            // 自定义条件需要外部注册的评估函数来分派。
            // 此处返回 Failed 以触发调用方注册自定义评估器。
            ConditionResult::failed("custom condition evaluator not registered")
        }
    }
}

/// 评估 TagRequirement 条件。
///
/// 领域规则 §5.1.2：检查目标实体的标签集合，验证 Has/Not 条件。
/// §3.2：标签 ID 对应的标签定义不存在时视为 Failed。
fn evaluate_tag_requirement(
    mode: TagRequirementMode,
    tag_id: &str,
    context: &ConditionContext,
) -> ConditionResult {
    let tags = match &context.tag_ids {
        Some(tags) => tags,
        None => {
            return ConditionResult::failed(format!(
                "tag context unavailable for tag '{}'",
                tag_id
            ));
        }
    };

    let has_tag = tags.iter().any(|t| t == tag_id);

    match mode {
        TagRequirementMode::Has => {
            if has_tag {
                ConditionResult::passed()
            } else {
                ConditionResult::failed(format!("entity does not have tag '{}'", tag_id))
            }
        }
        TagRequirementMode::Not => {
            if has_tag {
                ConditionResult::failed(format!("entity has forbidden tag '{}'", tag_id))
            } else {
                ConditionResult::passed()
            }
        }
    }
}

/// 评估 AttributeCheck 条件。
///
/// 领域规则 §5.1.3：读取目标实体的当前属性值，验证是否满足阈值。
/// §3.2：引用的属性不存在时视为 Failed。
fn evaluate_attribute_check(
    attribute_id: &str,
    operator: crate::core::capabilities::condition::foundation::ComparisonOp,
    threshold: f32,
    context: &ConditionContext,
) -> ConditionResult {
    let value = match context.attribute_values.get(attribute_id) {
        Some(v) => *v,
        None => {
            return ConditionResult::failed(format!(
                "attribute '{}' not found in context",
                attribute_id
            ));
        }
    };

    if operator.evaluate(value, threshold) {
        ConditionResult::passed()
    } else {
        ConditionResult::failed(format!(
            "attribute '{}' value {} does not satisfy {:?} {}",
            attribute_id, value, operator, threshold
        ))
    }
}

/// 评估 ResourceCheck 条件。
///
/// 领域规则 §5.1.4：读取目标实体的当前资源属性值，验证是否 >= 所需量。
/// ResourceCheck 是 AttributeCheck 的特化（使用 GreaterOrEqual 比较）。
fn evaluate_resource_check(
    resource_id: &str,
    required_amount: f32,
    context: &ConditionContext,
) -> ConditionResult {
    let value = match context.attribute_values.get(resource_id) {
        Some(v) => *v,
        None => {
            return ConditionResult::failed(format!(
                "resource '{}' not found in context",
                resource_id
            ));
        }
    };

    if value >= required_amount {
        ConditionResult::passed()
    } else {
        ConditionResult::failed(format!(
            "resource '{}' ({}) < required ({})",
            resource_id, value, required_amount
        ))
    }
}

/// 评估 AND 组合条件。
///
/// 领域规则 §5.2.1：短路评估——任一失败立即返回 Failed。
fn evaluate_and(children: &[Condition], context: &ConditionContext) -> ConditionResult {
    if children.is_empty() {
        // 空 AND 视为通过（数学惯例）
        return ConditionResult::passed();
    }

    for child in children {
        let result = evaluate(child, context);
        if !result.is_passed() {
            return result;
        }
    }
    ConditionResult::passed()
}

/// 评估 OR 组合条件。
///
/// 领域规则 §5.2.2：短路评估——任一通过立即返回 Passed。
fn evaluate_or(children: &[Condition], context: &ConditionContext) -> ConditionResult {
    if children.is_empty() {
        // 空 OR 视为不通过
        return ConditionResult::failed("empty OR group has no passing condition");
    }

    for child in children {
        let result = evaluate(child, context);
        if result.is_passed() {
            return ConditionResult::passed();
        }
    }
    ConditionResult::failed("no condition in OR group passed")
}

/// 免疫检查（领域规则 §5.3 特殊流程）。
///
/// 构建免疫检查条件并评估：目标是否具有 Tag.Immune.{effect_type}。
/// 免疫条件具有最高优先级（不变量 §3.5）。
pub fn check_immunity(context: &ConditionContext, effect_type: &str) -> ConditionResult {
    let immune_tag = format!("Immune.{}", effect_type);

    let condition = Condition::TagRequirement {
        mode: TagRequirementMode::Has,
        tag_id: immune_tag.clone(),
    };

    let result = evaluate(&condition, context);
    if result.is_passed() {
        ConditionResult::failed(format!("target is immune to '{}'", effect_type))
    } else {
        ConditionResult::passed()
    }
}
