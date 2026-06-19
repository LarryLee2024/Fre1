//! Aggregator 计算管线 — 纯函数四阶段执行器
//!
//! 严格按 Add → Multiply → Override → Clamp 顺序执行。
//! 无外部状态依赖，保证确定性。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::aggregator::events::AggregationComplete;
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
    entity: Entity,
    commands: &mut Commands,
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

    let result = AggregationResult {
        frame,
        attribute_id: attribute_id.to_string(),
        stage_values,
        participating_count: attr_modifiers.len(),
        was_overridden,
        final_value: current,
        base_value,
    };

    commands.trigger(AggregationComplete {
        entity,
        attribute_id: attribute_id.to_string(),
        final_value: result.final_value,
        base_value: result.base_value,
        frame: result.frame,
    });

    Ok(result)
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
