use crate::core::capabilities::aggregator::foundation::AggregationResult;
use crate::core::capabilities::aggregator::mechanism::{
    AggregatorState, clear_dirty, collect_dirty_attributes, initialize_state, mark_dirty,
    on_aggregation_complete,
};

#[test]
fn mark_dirty_adds_successfully() {
    let mut state = AggregatorState::empty();
    mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
    assert!(state.is_dirty("attr_000001"));
    assert_eq!(state.last_aggregation_frame, 1);
}

#[test]
fn mark_dirty_idempotent() {
    let mut state = AggregatorState::empty();
    mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
    mark_dirty(&mut state, "attr_000001", "mod_000002", 2);
    // still dirty, one entry
    assert_eq!(state.dirty_attributes.len(), 1);
}

#[test]
fn clear_dirty_succeeds() {
    let mut state = AggregatorState::empty();
    mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
    clear_dirty(&mut state, "attr_000001");
    assert!(!state.is_dirty("attr_000001"));
}

#[test]
fn collect_dirty_attributes_list() {
    let mut state = AggregatorState::empty();
    mark_dirty(&mut state, "attr_000003", "", 1);
    mark_dirty(&mut state, "attr_000001", "", 1);
    let dirty = collect_dirty_attributes(&state);
    assert_eq!(dirty, vec!["attr_000001", "attr_000003"]);
}

#[test]
fn dirty_attribute_cache_returns_none() {
    let mut state = AggregatorState::empty();
    state.cached_values.insert("attr_000001".to_string(), 42.0);
    assert_eq!(state.get_cached("attr_000001"), Some(42.0));
    mark_dirty(&mut state, "attr_000001", "", 1);
    assert_eq!(state.get_cached("attr_000001"), None);
}

#[test]
fn aggregation_complete_updates_cache() {
    let mut state = AggregatorState::empty();
    mark_dirty(&mut state, "attr_000001", "mod_000001", 1);
    let result = AggregationResult::new("attr_000001".to_string(), 10.0, 25.0, 1);
    on_aggregation_complete(&mut state, &result);
    assert!(!state.is_dirty("attr_000001"));
    assert_eq!(state.cached_values.get("attr_000001"), Some(&25.0));
    assert_eq!(state.aggregation_count, 1);
}

#[test]
fn test_initialize_state() {
    let ids = vec!["attr_000001".to_string(), "attr_000002".to_string()];
    let state = initialize_state(&ids);
    assert_eq!(state.cached_values.len(), 2);
    assert_eq!(state.cached_values.get("attr_000001"), Some(&0.0));
}

#[test]
fn has_dirty_returns_true_when_dirty() {
    let mut state = AggregatorState::empty();
    assert!(!state.has_dirty());
    mark_dirty(&mut state, "attr_000001", "", 1);
    assert!(state.has_dirty());
}
