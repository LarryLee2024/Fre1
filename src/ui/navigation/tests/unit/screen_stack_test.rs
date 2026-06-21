//! ScreenStack — LIFO navigation stack unit tests
//!
//! Tests verify the ScreenStack contract:
//! - New stack is empty (zero elements)
//! - push adds to top; duplicate top is a no-op
//! - pop returns the top element but preserves the root (last element)
//! - replace swaps the top element; on empty, degrades to push
//! - contains checks screen existence
//! - clear empties all elements
//! - iter returns elements bottom-to-top
//!
//! These are pure unit tests — ScreenStack is a plain struct with no ECS
//! dependencies.

use crate::ui::navigation::{ScreenStack, ScreenType};

#[test]
fn new_stack_is_empty() {
    let stack = ScreenStack::new();
    assert!(stack.is_empty(), "new stack must be empty");
    assert_eq!(stack.len(), 0, "new stack must have length 0");
}

#[test]
fn push_adds_element_to_stack() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    assert_eq!(stack.peek(), Some(&ScreenType::MainMenu));
    assert_eq!(stack.len(), 1);
}

#[test]
fn push_duplicate_top_is_noop() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::Battle);
    stack.push(ScreenType::Battle); // duplicate of current top
    assert_eq!(
        stack.len(),
        1,
        "duplicate top push must not increase stack size"
    );
    assert_eq!(stack.peek(), Some(&ScreenType::Battle));
}

#[test]
fn push_allows_different_screens() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.peek(), Some(&ScreenType::Battle));
}

#[test]
fn push_duplicate_non_top_is_allowed() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    stack.push(ScreenType::MainMenu); // MainMenu is not top (Battle is), so this adds
    assert_eq!(
        stack.len(),
        3,
        "duplicate non-top push must increase stack size"
    );
}

#[test]
fn pop_returns_top_element() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    assert_eq!(stack.pop(), Some(ScreenType::Battle));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.peek(), Some(&ScreenType::MainMenu));
}

#[test]
fn pop_single_element_returns_none() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    let result = stack.pop();
    assert_eq!(result, None, "pop on single-element stack must return None");
    assert_eq!(stack.len(), 1, "root screen must be preserved after pop");
}

#[test]
fn pop_empty_stack_returns_none() {
    let mut stack = ScreenStack::new();
    assert_eq!(stack.pop(), None, "pop on empty stack must return None");
}

#[test]
fn replace_swaps_top_element() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    let old = stack.replace(ScreenType::Inventory);
    assert_eq!(
        old,
        Some(ScreenType::Battle),
        "replace must return the old top"
    );
    assert_eq!(
        stack.peek(),
        Some(&ScreenType::Inventory),
        "replace must set new top"
    );
    assert_eq!(stack.len(), 2, "replace must not change stack depth");
}

#[test]
fn replace_on_empty_degrads_to_push() {
    let mut stack = ScreenStack::new();
    let old = stack.replace(ScreenType::MainMenu);
    assert_eq!(old, None, "replace on empty must return None");
    assert_eq!(
        stack.peek(),
        Some(&ScreenType::MainMenu),
        "replace on empty must push"
    );
    assert_eq!(stack.len(), 1);
}

#[test]
fn contains_returns_true_for_pushed_screen() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::Battle);
    assert!(stack.contains(ScreenType::Battle));
}

#[test]
fn contains_returns_false_for_unknown_screen() {
    let stack = ScreenStack::new();
    assert!(!stack.contains(ScreenType::Settings));
}

#[test]
fn contains_returns_false_after_pop() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    stack.pop();
    assert!(
        !stack.contains(ScreenType::Battle),
        "popped screen must not appear in contains"
    );
}

#[test]
fn clear_removes_all_elements() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    stack.clear();
    assert!(stack.is_empty(), "stack must be empty after clear");
    assert_eq!(stack.len(), 0);
}

#[test]
fn iter_returns_elements_bottom_to_top() {
    let mut stack = ScreenStack::new();
    stack.push(ScreenType::MainMenu);
    stack.push(ScreenType::Battle);
    stack.push(ScreenType::Inventory);
    let collected: Vec<&ScreenType> = stack.iter().collect();
    assert_eq!(
        collected,
        vec![
            &ScreenType::MainMenu,
            &ScreenType::Battle,
            &ScreenType::Inventory
        ],
        "iter must return elements in insertion order (bottom to top)"
    );
}

#[test]
fn default_is_equivalent_to_new() {
    let stack = ScreenStack::default();
    assert!(stack.is_empty(), "Default must produce an empty stack");
}
