//! Aggregator 计算管线 — 纯函数四阶段执行器
//!
//! 严格按 Add → Multiply → Override → Clamp 顺序执行。
//! 无外部状态依赖，保证确定性。

use std::collections::HashMap;

use crate::core::capabilities::aggregator::foundation::{
    AggregationResult, CalcPipeline, CalcStage, ModifierEntry, ModifierOp, PipelineError,
    default_stages,
};

/// 执行四阶段属性聚合计算。
///
/// # 参数
/// - `attribute_id` — 目标属性标识
/// - `base_value` — 属性基础值（来自 Attribute）
/// - `modifiers` — 该属性上的所有活跃修改器
/// - `pipeline` — 管线配置（默认使用 DEFAULT_PIPELINE）
/// - `min_value` — Clamp 下限（来自 AttributeDefinition 或管线覆盖）
/// - `max_value` — Clamp 上限
/// - `frame` — 当前帧号
///
/// # 返回
/// - `Ok(AggregationResult)` — 聚合结果
/// - `Err(PipelineError)` — 管线执行错误
pub fn execute_aggregation(
    attribute_id: &str,
    base_value: f32,
    modifiers: &[ModifierEntry],
    pipeline: &CalcPipeline,
    min_value: f32,
    max_value: f32,
    frame: u64,
) -> Result<AggregationResult, PipelineError> {
    // ── 1. 按属性过滤 + 按 ModifierOp 分组 ──
    let attr_modifiers: Vec<&ModifierEntry> = modifiers
        .iter()
        .filter(|m| m.target_attribute == attribute_id)
        .collect();

    // ── 2. 确定启用阶段 ──
    let stages = if pipeline.enabled_stages.is_empty() {
        default_stages()
    } else {
        pipeline.enabled_stages.clone()
    };

    let mut stage_values: HashMap<CalcStage, f32> = HashMap::new();
    let mut current = base_value;
    let mut was_overridden = false;

    // ── 3. 按阶段顺序执行 ──
    for stage in &stages {
        match stage {
            CalcStage::Add => {
                let add_modifiers = filter_by_op(&attr_modifiers, ModifierOp::Add);
                if !add_modifiers.is_empty() {
                    let sorted = sort_by_priority(add_modifiers, pipeline.priority_ascending);
                    let sum: f32 = sorted.iter().map(|m| m.magnitude).sum();
                    current = base_value + sum;
                }
                stage_values.insert(CalcStage::Add, current);
            }

            CalcStage::Multiply => {
                let mul_modifiers = filter_by_op(&attr_modifiers, ModifierOp::Multiply);
                if !mul_modifiers.is_empty() {
                    let sorted = sort_by_priority(mul_modifiers, pipeline.priority_ascending);
                    // 连乘: 1.2 × 1.3 = 1.56 (非 1 + 0.2 + 0.3 = 1.5)
                    let product: f32 = sorted.iter().map(|m| m.magnitude).product();
                    // 乘法 modifier 的 magnitude 表示倍率: 1.2 = +20%
                    current *= product;
                }
                stage_values.insert(CalcStage::Multiply, current);
            }

            CalcStage::Override => {
                let override_modifiers = filter_by_op(&attr_modifiers, ModifierOp::Override);
                if !override_modifiers.is_empty() {
                    let sorted = sort_by_priority(override_modifiers, pipeline.priority_ascending);
                    // 取优先级最高的 Override 值
                    current = sorted[0].magnitude;
                    was_overridden = true;
                }
                stage_values.insert(CalcStage::Override, current);
            }

            CalcStage::Clamp => {
                // 确定 clamp 边界：管线覆盖优先，否则用 min/max 参数
                let (lo, hi) = pipeline.clamp_override.unwrap_or((min_value, max_value));
                if lo > hi {
                    return Err(PipelineError::InvalidClampBounds { min: lo, max: hi });
                }
                current = current.clamp(lo, hi);
                stage_values.insert(CalcStage::Clamp, current);
            }
        }
    }

    Ok(AggregationResult {
        frame,
        attribute_id: attribute_id.to_string(),
        stage_values,
        participating_count: attr_modifiers.len(),
        was_overridden,
        final_value: current,
        base_value,
    })
}

/// 按 ModifierOp 过滤修改器列表。
fn filter_by_op<'a>(modifiers: &[&'a ModifierEntry], op: ModifierOp) -> Vec<&'a ModifierEntry> {
    modifiers.iter().filter(|m| m.op == op).copied().collect()
}

/// 按优先级排序。
fn sort_by_priority(mut modifiers: Vec<&ModifierEntry>, ascending: bool) -> Vec<&ModifierEntry> {
    if ascending {
        modifiers.sort_by_key(|m| m.priority);
    } else {
        modifiers.sort_by_key(|m| std::cmp::Reverse(m.priority));
    }
    modifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(op: ModifierOp, magnitude: f32, priority: u8, target: &str) -> ModifierEntry {
        ModifierEntry {
            op,
            magnitude,
            priority,
            target_attribute: target.to_string(),
        }
    }

    fn default_pipeline_for(attr: &str) -> CalcPipeline {
        CalcPipeline {
            attribute_id: attr.to_string(),
            enabled_stages: default_stages(),
            priority_ascending: true,
            clamp_override: None,
            cycle_detection: true,
        }
    }

    #[test]
    fn unit_001_base_value_unchanged_without_modifiers() {
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &[],
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        assert_eq!(result.final_value, 10.0);
        assert_eq!(result.base_value, 10.0);
        assert_eq!(result.participating_count, 0);
    }

    #[test]
    fn unit_002_single_add_modifier() {
        let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        assert_eq!(result.final_value, 15.0);
    }

    #[test]
    fn unit_003_multiple_add_modifiers_sum() {
        let modifiers = vec![
            make_entry(ModifierOp::Add, 3.0, 50, "attr_000001"),
            make_entry(ModifierOp::Add, 7.0, 60, "attr_000001"),
        ];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 10 + (3 + 7) = 20
        assert_eq!(result.final_value, 20.0);
        assert_eq!(result.participating_count, 2);
    }

    #[test]
    fn unit_004_multiply_is_compound_not_additive() {
        let modifiers = vec![
            make_entry(ModifierOp::Multiply, 1.2, 50, "attr_000001"),
            make_entry(ModifierOp::Multiply, 1.3, 60, "attr_000001"),
        ];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 10 × 1.2 × 1.3 = 15.6 (连乘，非加法)
        assert!((result.final_value - 15.6).abs() < 1e-5);
    }

    #[test]
    fn unit_005_override_takes_highest_priority() {
        let modifiers = vec![
            make_entry(ModifierOp::Override, 50.0, 80, "attr_000001"),
            make_entry(ModifierOp::Override, 99.0, 10, "attr_000001"),
        ];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 优先级 10 比 80 更优先（ascending = true）
        assert_eq!(result.final_value, 99.0);
        assert!(result.was_overridden);
    }

    #[test]
    fn unit_006_override_skipped_if_none() {
        let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 10 + 5 = 15, no override
        assert_eq!(result.final_value, 15.0);
        assert!(!result.was_overridden);
    }

    #[test]
    fn unit_007_clamp_lower_bound() {
        let modifiers = vec![make_entry(ModifierOp::Add, -50.0, 50, "attr_000001")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 10 - 50 = -40 → clamped to 0
        assert_eq!(result.final_value, 0.0);
    }

    #[test]
    fn unit_008_clamp_upper_bound() {
        let modifiers = vec![make_entry(ModifierOp::Add, 200.0, 50, "attr_000001")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // 10 + 200 = 210 → clamped to 100
        assert_eq!(result.final_value, 100.0);
    }

    #[test]
    fn unit_009_clamp_override_used_when_provided() {
        let mut pipeline = default_pipeline_for("attr_000001");
        pipeline.clamp_override = Some((5.0, 50.0));
        let modifiers = vec![make_entry(ModifierOp::Add, 200.0, 50, "attr_000001")];
        let result =
            execute_aggregation("attr_000001", 10.0, &modifiers, &pipeline, 0.0, 100.0, 1).unwrap();
        // clamp_override (5, 50) overrides min=0, max=100
        assert_eq!(result.final_value, 50.0);
    }

    #[test]
    fn unit_010_invalid_clamp_bounds_error() {
        let mut pipeline = default_pipeline_for("attr_000001");
        pipeline.clamp_override = Some((100.0, 0.0));
        let result = execute_aggregation("attr_000001", 10.0, &[], &pipeline, 0.0, 100.0, 1);
        assert!(matches!(
            result,
            Err(PipelineError::InvalidClampBounds { .. })
        ));
    }

    #[test]
    fn unit_011_unrelated_modifiers_ignored() {
        let modifiers = vec![make_entry(ModifierOp::Add, 99.0, 50, "attr_000002")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        assert_eq!(result.final_value, 10.0);
        assert_eq!(result.participating_count, 0);
    }

    #[test]
    fn unit_012_full_pipeline_add_multiply_override_clamp() {
        let modifiers = vec![
            make_entry(ModifierOp::Add, 10.0, 50, "attr_000001"),
            make_entry(ModifierOp::Multiply, 1.5, 50, "attr_000001"),
            make_entry(ModifierOp::Override, 30.0, 99, "attr_000001"),
        ];
        let result = execute_aggregation(
            "attr_000001",
            5.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // Add: 5 + 10 = 15
        // Multiply: 15 × 1.5 = 22.5
        // Override: 30 (priority 99, only one)
        // Clamp: 30 ∈ [0, 100]
        assert_eq!(result.final_value, 30.0);
    }

    #[test]
    fn unit_013_stage_values_tracked() {
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &[],
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            42,
        )
        .unwrap();
        assert_eq!(result.frame, 42);
        assert!(result.stage_values.contains_key(&CalcStage::Clamp));
    }

    #[test]
    fn unit_014_descending_priority_order() {
        let mut pipeline = default_pipeline_for("attr_000001");
        pipeline.priority_ascending = false;
        let modifiers = vec![
            make_entry(ModifierOp::Override, 10.0, 10, "attr_000001"),
            make_entry(ModifierOp::Override, 99.0, 99, "attr_000001"),
        ];
        let result =
            execute_aggregation("attr_000001", 5.0, &modifiers, &pipeline, 0.0, 100.0, 1).unwrap();
        // descending = true: higher value = more priority → 99 wins
        assert_eq!(result.final_value, 99.0);
    }

    #[test]
    fn unit_015_multiply_skip_if_no_multiply_modifiers() {
        let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
        let result = execute_aggregation(
            "attr_000001",
            10.0,
            &modifiers,
            &default_pipeline_for("attr_000001"),
            0.0,
            100.0,
            1,
        )
        .unwrap();
        // Add: 15, Multiply skipped: 15, Override skipped: 15, Clamp: 15
        assert_eq!(result.final_value, 15.0);
    }
}
