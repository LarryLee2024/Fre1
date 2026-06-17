use bevy::prelude::{KeyCode, MouseButton};

use crate::infra::input::action::{InputAction, InputMap};

#[test]
fn default_input_map_has_keyboard_bindings() {
    let map = InputMap::default();

    // Direction keys — WASD
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyW),
        Some(InputAction::MoveUp)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyS),
        Some(InputAction::MoveDown)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyA),
        Some(InputAction::MoveLeft)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyD),
        Some(InputAction::MoveRight)
    );

    // Confirm/cancel
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Space),
        Some(InputAction::Select)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Enter),
        Some(InputAction::Select)
    );

    // Camera
    assert_eq!(
        map.get_keyboard_action(&KeyCode::ArrowUp),
        Some(InputAction::CameraUp)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::ArrowDown),
        Some(InputAction::CameraDown)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Equal),
        Some(InputAction::CameraZoomIn)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Minus),
        Some(InputAction::CameraZoomOut)
    );

    // Meta commands
    assert_eq!(
        map.get_keyboard_action(&KeyCode::F5),
        Some(InputAction::QuickSave)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::F9),
        Some(InputAction::QuickLoad)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyT),
        Some(InputAction::EndTurn)
    );

    // Skill slots
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Digit1),
        Some(InputAction::SkillSlot1)
    );
    assert_eq!(
        map.get_keyboard_action(&KeyCode::Digit4),
        Some(InputAction::SkillSlot4)
    );
}

#[test]
fn default_input_map_has_mouse_bindings() {
    let map = InputMap::default();

    assert_eq!(
        map.get_mouse_action(&MouseButton::Left),
        Some(InputAction::Select)
    );
    assert_eq!(
        map.get_mouse_action(&MouseButton::Right),
        Some(InputAction::Cancel)
    );
}

#[test]
fn default_input_map_returns_none_for_unbound_keys() {
    let map = InputMap::default();

    assert_eq!(map.get_keyboard_action(&KeyCode::F1), None);
    assert_eq!(map.get_keyboard_action(&KeyCode::KeyZ), None);
    assert_eq!(map.get_mouse_action(&MouseButton::Middle), None);
}

#[test]
fn input_map_custom_bindings_override_defaults() {
    let mut map = InputMap::default();
    // Rebind Space to Cancel
    map.keyboard.insert(KeyCode::Space, InputAction::Cancel);

    assert_eq!(
        map.get_keyboard_action(&KeyCode::Space),
        Some(InputAction::Cancel)
    );
    // Other bindings unchanged
    assert_eq!(
        map.get_keyboard_action(&KeyCode::KeyW),
        Some(InputAction::MoveUp)
    );
}
