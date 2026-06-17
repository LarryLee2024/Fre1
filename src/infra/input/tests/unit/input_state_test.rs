use bevy::prelude::Vec2;

use crate::infra::input::action::InputAction;
use crate::infra::input::resources::InputState;

fn make_state() -> InputState {
    let mut state = InputState::default();
    state.pressed_actions = vec![InputAction::MoveUp, InputAction::Select];
    state.just_pressed_actions = vec![InputAction::Select];
    state.just_released_actions = vec![InputAction::Cancel];
    state.mouse_position = Vec2::new(100.0, 200.0);
    state.mouse_grid_pos = Some((5, 3));
    state
}

#[test]
fn input_state_clear_frame_removes_transient_state() {
    let mut state = make_state();

    state.clear_frame();

    assert!(
        state.pressed_actions.is_empty(),
        "pressed actions should be cleared"
    );
    assert!(
        state.just_pressed_actions.is_empty(),
        "just_pressed actions should be cleared"
    );
    assert!(
        state.just_released_actions.is_empty(),
        "just_released actions should be cleared"
    );
    assert!(
        state.mouse_grid_pos.is_none(),
        "mouse_grid_pos should be cleared"
    );
    // mouse_position persists (not transient)
    assert_eq!(state.mouse_position, Vec2::new(100.0, 200.0));
}

#[test]
fn input_state_just_pressed_returns_true_for_active_action() {
    let state = make_state();
    assert!(state.just_pressed(InputAction::Select));
    assert!(!state.just_pressed(InputAction::Cancel));
    assert!(!state.just_pressed(InputAction::MoveRight));
}

#[test]
fn input_state_just_released_returns_true_for_active_action() {
    let state = make_state();
    assert!(state.just_released(InputAction::Cancel));
    assert!(!state.just_released(InputAction::Select));
    assert!(!state.just_released(InputAction::MoveUp));
}

#[test]
fn input_state_pressed_returns_true_for_held_action() {
    let state = make_state();
    assert!(state.pressed(InputAction::MoveUp));
    assert!(state.pressed(InputAction::Select));
    assert!(!state.pressed(InputAction::MoveRight));
}

#[test]
fn input_state_default_is_empty() {
    let state = InputState::default();
    assert!(state.pressed_actions.is_empty());
    assert!(state.just_pressed_actions.is_empty());
    assert!(state.just_released_actions.is_empty());
    assert_eq!(state.mouse_position, Vec2::ZERO);
    assert!(state.mouse_grid_pos.is_none());
}
