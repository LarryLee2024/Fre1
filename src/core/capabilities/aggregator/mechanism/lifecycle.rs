//! Aggregator 生命周期管理
//!
//! 脏标记、批量重算编排、与 Modifier 容器的桥接。

use bevy::prelude::*;

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
) {
    state.dirty_attributes.insert(attribute_id.to_string());
    state.last_aggregation_frame = frame;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_mark_dirty_adds_attribute() {
        let mut state = AggregatorState::empty();
        mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
        assert!(state.is_dirty("attr_000001"));
        assert_eq!(state.last_aggregation_frame, 1);
    }

    #[test]
    fn unit_002_mark_dirty_is_idempotent() {
        let mut state = AggregatorState::empty();
        mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
        mark_dirty(&mut state, "attr_000001", "mod_000002", 2);
        // still dirty, one entry
        assert_eq!(state.dirty_attributes.len(), 1);
    }

    #[test]
    fn unit_003_clear_dirty_removes_attribute() {
        let mut state = AggregatorState::empty();
        mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
        clear_dirty(&mut state, "attr_000001");
        assert!(!state.is_dirty("attr_000001"));
    }

    #[test]
    fn unit_004_collect_dirty_attributes() {
        let mut state = AggregatorState::empty();
        mark_dirty(&mut state, "attr_000003", "", 1);
        mark_dirty(&mut state, "attr_000001", "", 1);
        let dirty = collect_dirty_attributes(&state);
        assert_eq!(dirty, vec!["attr_000001", "attr_000003"]);
    }

    #[test]
    fn unit_005_get_cached_returns_none_when_dirty() {
        let mut state = AggregatorState::empty();
        state.cached_values.insert("attr_000001".to_string(), 42.0);
        assert_eq!(state.get_cached("attr_000001"), Some(42.0));
        mark_dirty(&mut state, "attr_000001", "", 1);
        assert_eq!(state.get_cached("attr_000001"), None);
    }

    #[test]
    fn unit_006_on_aggregation_complete_updates_cache() {
        let mut state = AggregatorState::empty();
        mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
        let result = AggregationResult::new("attr_000001".to_string(), 10.0, 25.0, 1);
        on_aggregation_complete(&mut state, &result);
        assert!(!state.is_dirty("attr_000001"));
        assert_eq!(state.cached_values.get("attr_000001"), Some(&25.0));
        assert_eq!(state.aggregation_count, 1);
    }

    #[test]
    fn unit_007_initialize_state() {
        let ids = vec!["attr_000001".to_string(), "attr_000002".to_string()];
        let state = initialize_state(&ids);
        assert_eq!(state.cached_values.len(), 2);
        assert_eq!(state.cached_values.get("attr_000001"), Some(&0.0));
    }

    #[test]
    fn unit_008_has_dirty_true_when_dirty_exists() {
        let mut state = AggregatorState::empty();
        assert!(!state.has_dirty());
        mark_dirty(&mut state, "attr_000001", "", 1);
        assert!(state.has_dirty());
    }
}
