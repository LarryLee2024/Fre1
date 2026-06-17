use crate::core::domains::tactical::components::{GridPos, MovementPoints, MovementType};
use crate::core::domains::tactical::error::TacticalError;
use crate::core::domains::tactical::integration::movement::MP;
use crate::core::domains::tactical::resources::{GridLayout, GridMap, TileData, TileFlags};
use crate::core::domains::tactical::systems::movement_system::{
    MoveResult, validate_and_execute_move,
};

fn make_default_grid() -> GridMap {
    GridMap::new(10, 10, GridLayout::Square)
}

fn make_unit_mp() -> MovementPoints {
    MovementPoints::new(5.0, MovementType::Walk)
}

#[test]
fn validate_and_execute_move_moves_unit_to_valid_target() {
    let grid = make_default_grid();
    let mut mp = make_unit_mp();
    let mut pos = GridPos::new(0, 0);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(3, 0),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(result.is_ok());
    let move_result = result.unwrap();
    assert_eq!(move_result.new_pos, GridPos::new(3, 0));
    assert_eq!(pos, GridPos::new(3, 0), "unit position should be updated");
    assert!((move_result.cost.0 - 3.0).abs() < f32::EPSILON);
    assert!((move_result.remaining_mp.0 - 2.0).abs() < f32::EPSILON);
}

#[test]
fn validate_and_execute_move_out_of_bounds_returns_error() {
    let grid = make_default_grid();
    let mut mp = make_unit_mp();
    let mut pos = GridPos::new(0, 0);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(20, 20),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(matches!(result, Err(TacticalError::OutOfBounds)));
    assert_eq!(
        pos,
        GridPos::new(0, 0),
        "position should not change on error"
    );
}

#[test]
fn validate_and_execute_move_blocked_tile_returns_error() {
    let mut grid = make_default_grid();
    if let Some(tile) = grid.get_tile_mut(GridPos::new(3, 0)) {
        *tile = TileData::new(0, 0, TileFlags(0));
    }

    let mut mp = make_unit_mp();
    let mut pos = GridPos::new(0, 0);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(3, 0),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(matches!(result, Err(TacticalError::TileNotPassable)));
    assert_eq!(
        pos,
        GridPos::new(0, 0),
        "position should not change on error"
    );
}

#[test]
fn validate_and_execute_move_insufficient_mp_returns_error() {
    let grid = make_default_grid();
    let mut mp = MovementPoints::new(2.0, MovementType::Walk);
    let mut pos = GridPos::new(0, 0);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(5, 0),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(matches!(
        result,
        Err(TacticalError::InsufficientMovementPoints {
            required: 5.0,
            available: 2.0
        })
    ));
    assert_eq!(
        pos,
        GridPos::new(0, 0),
        "position should not change on error"
    );
}

#[test]
fn validate_and_execute_move_consumes_mp_correctly() {
    let grid = make_default_grid();
    let mut mp = MovementPoints::new(10.0, MovementType::Walk);
    let mut pos = GridPos::new(0, 0);

    let _ = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(4, 0),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(
        (mp.current - 6.0).abs() < f32::EPSILON,
        "should have 6 MP remaining"
    );
    assert!(
        (mp.consumed - 4.0).abs() < f32::EPSILON,
        "should have consumed 4 MP"
    );
}

#[test]
fn validate_and_execute_move_diagonal_uses_manhattan_distance() {
    let grid = make_default_grid();
    let mut mp = make_unit_mp();
    let mut pos = GridPos::new(0, 0);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(2, 3),
        &grid,
        &mut mp,
        &mut pos,
    );

    assert!(result.is_ok());
    assert!(
        (result.unwrap().cost.0 - 5.0).abs() < f32::EPSILON,
        "diagonal move should use manhattan distance"
    );
}

#[test]
fn validate_and_execute_move_returns_correct_move_result() {
    let grid = make_default_grid();
    let mut mp = make_unit_mp();
    let mut pos = GridPos::new(1, 1);

    let result = validate_and_execute_move(
        bevy::prelude::Entity::PLACEHOLDER,
        GridPos::new(4, 1),
        &grid,
        &mut mp,
        &mut pos,
    )
    .unwrap();

    assert_eq!(result.old_pos, GridPos::new(1, 1));
    assert_eq!(result.new_pos, GridPos::new(4, 1));
    assert!((result.cost.0 - 3.0).abs() < f32::EPSILON);
    assert!((result.remaining_mp.0 - 2.0).abs() < f32::EPSILON);
}
