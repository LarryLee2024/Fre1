use crate::core::domains::tactical::components::{MovementPoints, MovementType};

#[test]
fn movement_points_new_sets_current_to_max() {
    let mp = MovementPoints::new(5.0, MovementType::Walk);
    assert!((mp.current - 5.0).abs() < f32::EPSILON);
    assert!((mp.max - 5.0).abs() < f32::EPSILON);
    assert!((mp.consumed).abs() < f32::EPSILON);
    assert_eq!(mp.movement_type, MovementType::Walk);
}

#[test]
fn movement_points_consume_reduces_current() {
    let mut mp = MovementPoints::new(5.0, MovementType::Walk);
    let result = mp.consume(3.0);
    assert!(result, "consume should succeed with sufficient MP");
    assert!((mp.current - 2.0).abs() < f32::EPSILON);
    assert!((mp.consumed - 3.0).abs() < f32::EPSILON);
}

#[test]
fn movement_points_consume_returns_false_when_insufficient() {
    let mut mp = MovementPoints::new(5.0, MovementType::Walk);
    let result = mp.consume(10.0);
    assert!(!result, "consume should fail with insufficient MP");
    assert!(
        (mp.current - 5.0).abs() < f32::EPSILON,
        "current should not change on failure"
    );
    assert!(
        (mp.consumed).abs() < f32::EPSILON,
        "consumed should not change on failure"
    );
}

#[test]
fn movement_points_consume_exact_amount() {
    let mut mp = MovementPoints::new(5.0, MovementType::Walk);
    let result = mp.consume(5.0);
    assert!(result);
    assert!((mp.current).abs() < f32::EPSILON);
    assert!((mp.consumed - 5.0).abs() < f32::EPSILON);
}

#[test]
fn movement_points_reset_restores_full_mp() {
    let mut mp = MovementPoints::new(5.0, MovementType::Walk);
    mp.consume(3.0);
    mp.reset();
    assert!(
        (mp.current - 5.0).abs() < f32::EPSILON,
        "current should restore to max"
    );
    assert!(
        (mp.consumed).abs() < f32::EPSILON,
        "consumed should reset to zero"
    );
}

#[test]
fn movement_points_zero_cost_consume_succeeds() {
    let mut mp = MovementPoints::new(5.0, MovementType::Walk);
    let result = mp.consume(0.0);
    assert!(result);
    assert!((mp.current - 5.0).abs() < f32::EPSILON);
}
