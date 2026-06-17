use crate::shared::time::*;
#[test]
fn initial_value_is_zero() {
    let t = GameTime::new();
    assert_eq!(t.frame(), 0);
    assert_eq!(t.turn(), 0);
}

#[test]
fn advance_frame_increases() {
    let mut t = GameTime::new();
    t.advance_frame();
    assert_eq!(t.frame(), 1);
    assert_eq!(t.turn(), 0);
}

#[test]
fn advance_turn() {
    let mut t = GameTime::at(5, 0);
    t.advance_turn();
    assert_eq!(t.frame(), 5);
    assert_eq!(t.turn(), 1);
}

#[test]
fn at_constructor_works_correctly() {
    let t = GameTime::at(10, 3);
    assert_eq!(t.frame(), 10);
    assert_eq!(t.turn(), 3);
}

#[test]
fn display_format_correct() {
    let t = GameTime::at(42, 7);
    assert_eq!(t.to_string(), "F42_T7");
}

#[test]
fn value_equality() {
    let a = GameTime::at(5, 2);
    let b = GameTime::at(5, 2);
    let c = GameTime::at(5, 3);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn ordering_by_frame_and_turn() {
    let early = GameTime::at(1, 0);
    let mid = GameTime::at(1, 1);
    let late = GameTime::at(2, 0);
    assert!(early < mid);
    assert!(mid < late);
}

#[test]
fn display_is_not_empty() {
    let t = GameTime::new();
    assert!(!t.to_string().is_empty());
}
