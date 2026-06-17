use crate::core::capabilities::stacking::foundation::values::StackingState;

#[test]
fn stack_state_initial_state() {
    let state = StackingState::new(5).unwrap();
    assert_eq!(state.stack_count, 1);
    assert_eq!(state.max_stacks, 5);
    assert!(state.stack_members.is_empty());
}

#[test]
fn stack_state_rejects_zero_max_stacks() {
    let result = StackingState::new(0);
    assert!(result.is_err());
}

#[test]
fn stack_state_add_layers_succeeds() {
    let mut state = StackingState::new(5).unwrap();
    let added = state.add_layers(2);
    assert_eq!(added, 2);
    assert_eq!(state.stack_count, 3);
}

#[test]
fn stack_state_add_layers_reaches_limit() {
    let mut state = StackingState::new(3).unwrap();
    state.stack_count = 2;
    let added = state.add_layers(3);
    assert_eq!(added, 1); // only 1 slot remaining
    assert_eq!(state.stack_count, 3);
}

#[test]
fn stack_state_checks_if_at_max() {
    let mut state = StackingState::new(3).unwrap();
    assert!(!state.is_at_max());
    state.stack_count = 3;
    assert!(state.is_at_max());
}

#[test]
fn stack_state_remove_layers_succeeds() {
    let mut state = StackingState::new(5).unwrap();
    state.stack_count = 5;
    state.remove_layers(2);
    assert_eq!(state.stack_count, 3);
}

#[test]
fn stack_state_remove_layers_not_below_one() {
    let mut state = StackingState::new(5).unwrap();
    state.remove_layers(10);
    assert_eq!(state.stack_count, 1); // floor at 1
}

#[test]
fn stack_state_member_management() {
    let mut state = StackingState::new(5).unwrap();
    state.add_member("inst_001");
    state.add_member("inst_002");
    assert_eq!(state.stack_members.len(), 2);

    state.remove_member("inst_001");
    assert_eq!(state.stack_members.len(), 1);
}

#[test]
fn stack_state_remaining_capacity_calc() {
    let mut state = StackingState::new(5).unwrap();
    assert_eq!(state.remaining_capacity(), 4);
    state.stack_count = 4;
    assert_eq!(state.remaining_capacity(), 1);
    state.stack_count = 5;
    assert_eq!(state.remaining_capacity(), 0);
}

#[test]
fn stack_state_current_layers() {
    let state = StackingState::new(5).unwrap();
    assert_eq!(state.current_layers(), 1);
}
