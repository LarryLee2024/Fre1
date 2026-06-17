use crate::infra::input::action::InputAction;

#[test]
fn input_action_name_returns_correct_identifier() {
    let cases = [
        (InputAction::Select, "Select"),
        (InputAction::Cancel, "Cancel"),
        (InputAction::MoveUp, "MoveUp"),
        (InputAction::MoveDown, "MoveDown"),
        (InputAction::MoveLeft, "MoveLeft"),
        (InputAction::MoveRight, "MoveRight"),
        (InputAction::CameraUp, "CameraUp"),
        (InputAction::CameraDown, "CameraDown"),
        (InputAction::CameraLeft, "CameraLeft"),
        (InputAction::CameraRight, "CameraRight"),
        (InputAction::CameraZoomIn, "CameraZoomIn"),
        (InputAction::CameraZoomOut, "CameraZoomOut"),
        (InputAction::QuickSave, "QuickSave"),
        (InputAction::QuickLoad, "QuickLoad"),
        (InputAction::OpenMenu, "OpenMenu"),
        (InputAction::EndTurn, "EndTurn"),
        (InputAction::SkillSlot1, "SkillSlot1"),
        (InputAction::SkillSlot2, "SkillSlot2"),
        (InputAction::SkillSlot3, "SkillSlot3"),
        (InputAction::SkillSlot4, "SkillSlot4"),
    ];

    for (action, expected) in &cases {
        assert_eq!(
            action.name(),
            *expected,
            "InputAction::{:?} name mismatch",
            action
        );
    }
}

#[test]
fn input_action_all_variants_have_unique_names() {
    let mut names = std::collections::HashSet::new();
    let variants = [
        InputAction::Select,
        InputAction::Cancel,
        InputAction::MoveUp,
        InputAction::MoveDown,
        InputAction::MoveLeft,
        InputAction::MoveRight,
        InputAction::CameraUp,
        InputAction::CameraDown,
        InputAction::CameraLeft,
        InputAction::CameraRight,
        InputAction::CameraZoomIn,
        InputAction::CameraZoomOut,
        InputAction::QuickSave,
        InputAction::QuickLoad,
        InputAction::OpenMenu,
        InputAction::EndTurn,
        InputAction::SkillSlot1,
        InputAction::SkillSlot2,
        InputAction::SkillSlot3,
        InputAction::SkillSlot4,
    ];

    for variant in &variants {
        assert!(
            names.insert(variant.name()),
            "duplicate name for InputAction::{:?}",
            variant
        );
    }
}
