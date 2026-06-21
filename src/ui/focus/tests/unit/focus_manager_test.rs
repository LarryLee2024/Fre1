//! FocusManager — Global focus state management unit tests
//!
//! Tests verify the FocusManager contract:
//! - Initial state has no focused entity or active group
//! - focus() sets the entity, group_id, and group index
//! - blur() clears the focused entity and active group
//! - is_focused() correctly identifies the focused entity
//! - push_focus() / pop_focus() save and restore focus history
//! - activate_group() activates a group and attempts history restoration
//!
//! These are pure unit tests — FocusManager is a plain struct with no ECS
//! dependencies beyond the Bevy Entity type.

use bevy::prelude::Entity;
use crate::ui::focus::FocusManager;

/// Helper to create a mock Entity for testing
fn mock_entity(id: u64) -> Entity {
    Entity::from_bits(id)
}

#[test]
fn initial_state_has_no_focus() {
    let fm = FocusManager::default();
    assert!(fm.focused_entity.is_none(), "initial focused_entity must be None");
    assert!(fm.active_group.is_none(), "initial active_group must be None");
    assert!(fm.group_indices.is_empty(), "initial group_indices must be empty");
    assert!(fm.focus_history.is_empty(), "initial focus_history must be empty");
}

#[test]
fn focus_sets_entity_and_group() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 3);
    assert_eq!(fm.focused_entity, Some(entity));
    assert_eq!(fm.active_group, Some(0));
    assert_eq!(fm.group_indices.get(&0), Some(&3));
}

#[test]
fn focus_updates_active_group() {
    let mut fm = FocusManager::default();
    fm.focus(mock_entity(1), 0, 0);
    assert_eq!(fm.active_group, Some(0));
    fm.focus(mock_entity(2), 1, 0);
    assert_eq!(fm.active_group, Some(1), "focus must update active_group");
}

#[test]
fn blur_clears_focus() {
    let mut fm = FocusManager::default();
    fm.focus(mock_entity(1), 0, 0);
    fm.blur();
    assert!(fm.focused_entity.is_none(), "focused_entity must be None after blur");
    assert!(fm.active_group.is_none(), "active_group must be None after blur");
}

#[test]
fn blur_does_not_clear_group_indices() {
    let mut fm = FocusManager::default();
    fm.focus(mock_entity(1), 0, 5);
    fm.blur();
    assert_eq!(
        fm.group_indices.get(&0),
        Some(&5),
        "blur must preserve group_indices"
    );
}

#[test]
fn is_focused_returns_true_for_focused_entity() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 0);
    assert!(fm.is_focused(entity));
}

#[test]
fn is_focused_returns_false_for_unfocused_entity() {
    let mut fm = FocusManager::default();
    fm.focus(mock_entity(1), 0, 0);
    assert!(!fm.is_focused(mock_entity(2)));
}

#[test]
fn is_focused_returns_false_after_blur() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 0);
    fm.blur();
    assert!(!fm.is_focused(entity));
}

#[test]
fn push_focus_saves_current_focus_to_history() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 0);
    fm.push_focus();
    assert_eq!(fm.focus_history.get(&0), Some(&entity));
}

#[test]
fn push_focus_noops_when_no_focus() {
    let mut fm = FocusManager::default();
    fm.push_focus(); // no focus set, should not panic
    assert!(fm.focus_history.is_empty(), "push_focus with no focus must not add to history");
}

#[test]
fn pop_focus_restores_saved_focus() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 0);
    fm.push_focus();
    fm.blur();
    let restored = fm.pop_focus(0);
    assert_eq!(restored, Some(entity), "pop_focus must return the saved entity");
    assert_eq!(fm.focused_entity, Some(entity), "focused_entity must be restored");
    assert_eq!(fm.active_group, Some(0), "active_group must be restored");
}

#[test]
fn pop_focus_returns_none_for_unknown_group() {
    let mut fm = FocusManager::default();
    let result = fm.pop_focus(99);
    assert_eq!(result, None, "pop_focus on unknown group must return None");
}

#[test]
fn pop_focus_removes_history_entry() {
    let mut fm = FocusManager::default();
    fm.focus(mock_entity(1), 0, 0);
    fm.push_focus();
    fm.pop_focus(0);
    assert!(
        !fm.focus_history.contains_key(&0),
        "pop_focus must remove the history entry"
    );
}

#[test]
fn activate_group_restores_history_when_available() {
    let mut fm = FocusManager::default();
    let entity = mock_entity(1);
    fm.focus(entity, 0, 0);
    fm.push_focus();
    fm.blur();
    fm.activate_group(0, Some((mock_entity(2), 5)));
    assert_eq!(
        fm.focused_entity,
        Some(entity),
        "activate_group must restore history over first_entity"
    );
    assert_eq!(fm.active_group, Some(0));
}

#[test]
fn activate_group_uses_first_entity_when_no_history() {
    let mut fm = FocusManager::default();
    let first = mock_entity(1);
    fm.activate_group(0, Some((first, 3)));
    assert_eq!(
        fm.focused_entity,
        Some(first),
        "activate_group without history must use first_entity"
    );
    assert_eq!(fm.group_indices.get(&0), Some(&3));
}

#[test]
fn activate_group_sets_active_group_without_focus() {
    let mut fm = FocusManager::default();
    fm.activate_group(0, None);
    assert_eq!(fm.active_group, Some(0));
    assert!(fm.focused_entity.is_none(), "no focus when no history and no first_entity");
}
