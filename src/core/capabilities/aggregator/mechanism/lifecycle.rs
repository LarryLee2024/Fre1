//! Aggregator 生命周期管理
//!
//! 脏标记、批量重算编排、与 Modifier 容器的桥接。

use bevy::prelude::*;

use crate::core::capabilities::aggregator::events::AggregateDirty;
use crate::core::capabilities::aggregator::foundation::AggregationResult;
use crate::core::capabilities::aggregator::mechanism::components::AggregatorState;

/// 标记指定属性为 Dirty。
///
/// 如果该属性已是 Dirty 状态则不做重复标记（幂等）。
pub fn mark_dirty(
    state: &mut AggregatorState,
    attribute_id: &str,
    trigger_source: &str,
    frame: u64,
    entity: Entity,
    commands: &mut Commands,
) {
    state.dirty_attributes.insert(attribute_id.to_string());
    state.last_aggregation_frame = frame;
    commands.trigger(AggregateDirty {
        entity,
        attribute_id: attribute_id.to_string(),
        trigger_source: trigger_source.to_string(),
    });
    // trigger_source 信息当前仅用于事件载荷，不在 state 中保留
    let _ = trigger_source;
}

/// 清除指定属性的 Dirty 标记。
pub fn clear_dirty(state: &mut AggregatorState, attribute_id: &str) {
    state.dirty_attributes.remove(attribute_id);
}

/// 批量收集脏属性列表（去重）。
pub fn collect_dirty_attributes(state: &AggregatorState) -> Vec<String> {
    let mut attrs: Vec<String> = state.dirty_attributes.iter().cloned().collect();
    attrs.sort();
    attrs
}

/// 聚合完成后更新缓存和状态。
pub fn on_aggregation_complete(state: &mut AggregatorState, result: &AggregationResult) {
    state
        .cached_values
        .insert(result.attribute_id.clone(), result.final_value);
    state.dirty_attributes.remove(&result.attribute_id);
    state.aggregation_count += 1;
}

/// 创建初始 AggregatorState 并预填充已知属性的基础值缓存。
pub fn initialize_state(attribute_ids: &[String]) -> AggregatorState {
    let mut state = AggregatorState::empty();
    for id in attribute_ids {
        state.cached_values.insert(id.clone(), 0.0);
    }
    state
}
