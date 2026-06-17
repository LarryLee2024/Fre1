//! 聚合器 ECS 系统 — Observer 模式
//!
//! 响应 AggregateDirty 事件：触发属性聚合管线，重新计算目标属性的最终值。

use bevy::prelude::*;

use crate::core::capabilities::aggregator::events::AggregateDirty;
use crate::core::capabilities::aggregator::foundation::{
    default_stages, CalcPipeline, ModifierEntry as AggregatorModifierEntry,
    ModifierOp as AggregatorModifierOp,
};
use crate::core::capabilities::aggregator::mechanism::pipeline::execute_aggregation;
use crate::core::capabilities::attribute::foundation::AttributeId;
use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::modifier::foundation::ModifierOp;
use crate::core::capabilities::modifier::mechanism::ModifierContainer;

/// 将 modifier 域的 ModifierOp 转换为 aggregator 域的 ModifierOp。
fn convert_op(op: &ModifierOp) -> AggregatorModifierOp {
    match op {
        ModifierOp::Add => AggregatorModifierOp::Add,
        ModifierOp::Multiply => AggregatorModifierOp::Multiply,
        ModifierOp::Override => AggregatorModifierOp::Override,
    }
}

/// 响应 `AggregateDirty` 事件：重新计算实体的指定属性值。
///
/// 流程：
/// 1. 从 ModifierContainer 中收集目标属性的所有修改器
/// 2. 执行四阶段聚合管线（Add → Multiply → Override → Clamp）
/// 3. 将计算结果写回 AttributeContainer 的 current_value
pub(crate) fn on_aggregate_dirty(
    trigger: On<AggregateDirty>,
    mut attr_query: Query<&mut AttributeContainer>,
    mod_query: Query<&ModifierContainer>,
) {
    let entity = trigger.entity;
    let attr_id_str = &trigger.event().attribute_id;

    let Ok(mut attr_container) = attr_query.get_mut(entity) else { return; };
    let Ok(mod_container) = mod_query.get(entity) else { return; };

    // 查找目标属性的定义（取 base_value）
    let attr_id = AttributeId::new(attr_id_str.as_str());
    let Some(attr_value) = attr_container.attributes.get(&attr_id) else { return; };
    let base_value = attr_value.base_value;

    // 收集该属性的所有修改器并转换到 aggregator 域
    let modifiers: Vec<AggregatorModifierEntry> = mod_container
        .modifiers
        .get(attr_id_str.as_str())
        .map(|mods| {
            mods.iter()
                .map(|m| AggregatorModifierEntry {
                    op: convert_op(&m.op),
                    magnitude: m.magnitude,
                    priority: m.priority,
                    target_attribute: m.target_attribute.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    // 使用默认管线执行聚合
    let pipeline = CalcPipeline {
        attribute_id: attr_id_str.clone(),
        enabled_stages: default_stages(),
        priority_ascending: true,
        clamp_override: None,
        cycle_detection: true,
    };

    // 执行聚合（使用宽松边界）
    let result = execute_aggregation(
        attr_id_str,
        base_value,
        &modifiers,
        &pipeline,
        f32::NEG_INFINITY,
        f32::INFINITY,
        0,
    );

    if let Ok(agg_result) = result {
        // 写回 current_value
        if let Some(val) = attr_container.attributes.get_mut(&attr_id) {
            val.current_value = agg_result.final_value;
        }
    }
}
