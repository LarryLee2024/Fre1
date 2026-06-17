//! Condition 评估器
//!
//! 纯函数条件评估，无副作用（领域规则 §3.1 不变量）。
//! 递归遍历 Condition 树，对叶子节点分发到对应检查逻辑。
//!
//! 详见 docs/02-domain/condition_domain.md §5.1-5.3。

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
///
/// # 用法
/// ```
/// # use fre::core::capabilities::condition::foundation::ConditionContext;
/// # use fre::core::capabilities::condition::mechanism::check_immunity;
/// let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
/// let result = check_immunity(&ctx, "Fire");
/// assert!(!result.is_passed()); // immune → check_immunity returns Failed
/// ```
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::core::capabilities::condition::foundation::{
        ComparisonOp, CustomCondition, CustomConditionId,
    };

    // ── TagRequirement ──────────────────────────────────────

    #[test]
    fn unit_001_tag_requirement_has_passes_when_tag_present() {
        let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into(), "Status.Stunned".into()]);
        let cond = Condition::TagRequirement {
            mode: TagRequirementMode::Has,
            tag_id: "Immune.Fire".into(),
        };
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_002_tag_requirement_has_fails_when_tag_absent() {
        let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
        let cond = Condition::TagRequirement {
            mode: TagRequirementMode::Has,
            tag_id: "Immune.Ice".into(),
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_003_tag_requirement_not_passes_when_tag_absent() {
        let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
        let cond = Condition::TagRequirement {
            mode: TagRequirementMode::Not,
            tag_id: "Immune.Ice".into(),
        };
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_004_tag_requirement_not_fails_when_tag_present() {
        let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
        let cond = Condition::TagRequirement {
            mode: TagRequirementMode::Not,
            tag_id: "Immune.Fire".into(),
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_005_tag_requirement_fails_when_context_missing() {
        let ctx = ConditionContext::empty();
        let cond = Condition::TagRequirement {
            mode: TagRequirementMode::Has,
            tag_id: "Immune.Fire".into(),
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    // ── AttributeCheck ──────────────────────────────────────

    #[test]
    fn unit_006_attribute_check_greater_passes() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 18.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        };
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_007_attribute_check_fails_on_threshold() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 12.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_008_attribute_check_fails_on_missing_attr() {
        let ctx = ConditionContext::with_attributes(HashMap::new());
        let cond = Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    // ── ResourceCheck ───────────────────────────────────────

    #[test]
    fn unit_009_resource_check_passes_when_sufficient() {
        let mut attrs = HashMap::new();
        attrs.insert("mana".into(), 50.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::ResourceCheck {
            resource_id: "mana".into(),
            required_amount: 30.0,
        };
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_010_resource_check_fails_when_insufficient() {
        let mut attrs = HashMap::new();
        attrs.insert("mana".into(), 20.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::ResourceCheck {
            resource_id: "mana".into(),
            required_amount: 30.0,
        };
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_011_resource_check_exact_amount_passes() {
        let mut attrs = HashMap::new();
        attrs.insert("mana".into(), 30.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::ResourceCheck {
            resource_id: "mana".into(),
            required_amount: 30.0,
        };
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    // ── AND / OR / NOT ──────────────────────────────────────

    #[test]
    fn unit_012_and_all_pass() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 18.0);
        attrs.insert("dex".into(), 14.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::And(vec![
            Condition::AttributeCheck {
                attribute_id: "str".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 15.0,
            },
            Condition::AttributeCheck {
                attribute_id: "dex".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 12.0,
            },
        ]);
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_013_and_one_fails() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 18.0);
        attrs.insert("dex".into(), 10.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::And(vec![
            Condition::AttributeCheck {
                attribute_id: "str".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 15.0,
            },
            Condition::AttributeCheck {
                attribute_id: "dex".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 12.0,
            },
        ]);
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_014_empty_and_passes() {
        let ctx = ConditionContext::empty();
        let cond = Condition::And(vec![]);
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_015_or_any_passes() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 10.0);
        attrs.insert("dex".into(), 14.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::Or(vec![
            Condition::AttributeCheck {
                attribute_id: "str".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 15.0,
            },
            Condition::AttributeCheck {
                attribute_id: "dex".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 8.0,
            },
        ]);
        // str=10 fails threshold 15, but dex=14 ≥ 8 → OR passes
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_016_or_all_fail() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 10.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::Or(vec![
            Condition::AttributeCheck {
                attribute_id: "str".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 15.0,
            },
            Condition::AttributeCheck {
                attribute_id: "dex".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 12.0,
            },
        ]);
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_017_empty_or_fails() {
        let ctx = ConditionContext::empty();
        let cond = Condition::Or(vec![]);
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_018_not_inverts_pass() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 18.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::Not(Box::new(Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::LessThan,
            threshold: 15.0,
        }));
        // str=18, LessThan 15 → false, NOT → true
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_019_not_inverts_fail() {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 10.0);
        let ctx = ConditionContext::with_attributes(attrs);
        let cond = Condition::Not(Box::new(Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        }));
        // str=10, GreaterOrEqual 15 → false, NOT → true
        assert!(evaluate(&cond, &ctx).is_passed());
    }

    // ── 复合嵌套 ────────────────────────────────────────────

    #[test]
    fn unit_020_nested_and_or_tree() {
        // (str >= 15 AND dex >= 12) OR (has_tag "Warrior")
        let ctx = {
            let mut attrs = HashMap::new();
            attrs.insert("str".into(), 10.0);
            attrs.insert("dex".into(), 14.0);
            ConditionContext {
                tag_ids: Some(vec![]),
                attribute_values: attrs,
            }
        };

        let cond = Condition::Or(vec![
            Condition::And(vec![
                Condition::AttributeCheck {
                    attribute_id: "str".into(),
                    operator: ComparisonOp::GreaterOrEqual,
                    threshold: 15.0,
                },
                Condition::AttributeCheck {
                    attribute_id: "dex".into(),
                    operator: ComparisonOp::GreaterOrEqual,
                    threshold: 12.0,
                },
            ]),
            Condition::TagRequirement {
                mode: TagRequirementMode::Has,
                tag_id: "Warrior".into(),
            },
        ]);

        // str=10, dex=14, no tag → AND fails, OR with tag fails too
        assert!(!evaluate(&cond, &ctx).is_passed());
    }

    #[test]
    fn unit_021_immune_check_no_immunity() {
        let ctx = ConditionContext::with_tags(vec![]);
        // no Immune.Fire tag → not immune → check_immunity returns Passed
        assert!(check_immunity(&ctx, "Fire").is_passed());
    }

    #[test]
    fn unit_022_immune_check_has_immunity() {
        let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
        // has Immune.Fire tag → immune → check_immunity returns Failed
        assert!(!check_immunity(&ctx, "Fire").is_passed());
    }

    // ── ComparisonOp ────────────────────────────────────────

    #[test]
    fn unit_023_comparison_equal() {
        assert!(ComparisonOp::Equal.evaluate(10.0, 10.0));
        assert!(!ComparisonOp::Equal.evaluate(10.0, 11.0));
    }

    #[test]
    fn unit_024_comparison_greater() {
        assert!(ComparisonOp::GreaterThan.evaluate(10.0, 5.0));
        assert!(!ComparisonOp::GreaterThan.evaluate(5.0, 10.0));
        assert!(!ComparisonOp::GreaterThan.evaluate(5.0, 5.0));
    }

    #[test]
    fn unit_025_comparison_less() {
        assert!(ComparisonOp::LessThan.evaluate(5.0, 10.0));
        assert!(!ComparisonOp::LessThan.evaluate(10.0, 5.0));
        assert!(!ComparisonOp::LessThan.evaluate(5.0, 5.0));
    }

    // ── Custom condition ────────────────────────────────────

    #[test]
    fn unit_026_custom_returns_failed_by_default() {
        let ctx = ConditionContext::empty();
        let cond = Condition::Custom(CustomCondition::new(CustomConditionId(42)));
        assert!(!evaluate(&cond, &ctx).is_passed());
    }
}
