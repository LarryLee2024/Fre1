//! Dirty<T> — ViewModel dirty-flag tracking unit tests
//!
//! Tests verify the Dirty<T> change-tracking contract:
//! - New Dirty<T> starts dirty (is_dirty = true)
//! - consume() returns true once and clears the flag
//! - get_mut() automatically marks dirty
//! - get() does NOT mark dirty (read-only access)
//! - mark_dirty() explicitly re-sets the flag
//! - Default creates dirty state
//!
//! These are pure unit tests with no ECS dependencies — Dirty<T> is a
//! self-contained wrapper type.

use crate::ui::binding::Dirty;

#[test]
fn new_is_dirty() {
    let mut dirty = Dirty::new(42);
    assert!(dirty.consume(), "new Dirty<T> must start in dirty state");
}

#[test]
fn consume_clears_flag() {
    let mut dirty = Dirty::new(42);
    dirty.consume();
    assert!(!dirty.consume(), "consume() must clear the dirty flag");
}

#[test]
fn consume_returns_true_only_once_per_mark() {
    let mut dirty = Dirty::new(42);
    assert!(dirty.consume(), "first consume() returns true");
    assert!(!dirty.consume(), "second consume() returns false");
    dirty.mark_dirty();
    assert!(dirty.consume(), "after re-mark, consume() returns true again");
    assert!(!dirty.consume(), "consumed again, returns false");
}

#[test]
fn get_mut_marks_dirty() {
    let mut dirty = Dirty::new(42);
    dirty.consume(); // clean the flag
    dirty.get_mut();
    assert!(dirty.consume(), "get_mut() must mark the component dirty");
}

#[test]
fn get_does_not_mark_dirty() {
    let mut dirty = Dirty::new(42);
    dirty.consume(); // clean the flag
    let _ = dirty.get();
    assert!(!dirty.consume(), "get() must NOT mark the component dirty");
}

#[test]
fn mark_dirty_sets_flag() {
    let mut dirty = Dirty::new(42);
    dirty.consume(); // clean the flag
    dirty.mark_dirty();
    assert!(dirty.consume(), "mark_dirty() must re-set the flag");
}

#[test]
fn default_creates_dirty_state() {
    let mut dirty: Dirty<i32> = Dirty::default();
    assert!(dirty.consume(), "Default must create Dirty with is_dirty = true");
}

#[test]
fn multiple_round_trip_cycles() {
    let mut dirty = Dirty::new(0);

    // Cycle 1: new → consume → mark → consume
    assert!(dirty.consume(), "cycle 1: initial dirty");
    assert!(!dirty.consume(), "cycle 1: consumed");
    dirty.mark_dirty();
    assert!(dirty.consume(), "cycle 1: after mark_dirty");

    // Cycle 2: get_mut → consume
    dirty.get_mut();
    assert!(dirty.consume(), "cycle 2: after get_mut");
    assert!(!dirty.consume(), "cycle 2: consumed");

    // Cycle 3: mark → get → consume (get should not interfere)
    dirty.consume(); // clear, skipping intermediate checks
    dirty.get_mut();
    let _ = dirty.get(); // read-only should not clear the flag
    assert!(dirty.consume(), "cycle 3: get() does not consume the flag");
}

#[test]
fn inner_value_accessible_via_get() {
    let dirty = Dirty::new(42);
    assert_eq!(*dirty.get(), 42, "get() must return the inner value");
}

#[test]
fn inner_value_accessible_via_get_mut() {
    let mut dirty = Dirty::new(42);
    *dirty.get_mut() = 100;
    assert_eq!(*dirty.get(), 100, "get_mut() must allow mutation of inner value");
}
