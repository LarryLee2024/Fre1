use crate::core::domains::tactical::components::{GridPos, MovementType};
use crate::core::domains::tactical::rules::movement::{movement_cost, path_total_cost};

#[test]
fn walk_on_plain_terrain_costs_one() {
    let cost = movement_cost(
        0,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!((cost - 1.0).abs() < f32::EPSILON);
}

#[test]
fn walk_on_road_is_half_cost() {
    let cost = movement_cost(
        1,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!((cost - 0.5).abs() < f32::EPSILON);
}

#[test]
fn walk_on_forest_is_double_cost() {
    let cost = movement_cost(
        2,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!((cost - 2.0).abs() < f32::EPSILON);
}

#[test]
fn walk_on_swamp_is_triple_cost() {
    let cost = movement_cost(
        3,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!((cost - 3.0).abs() < f32::EPSILON);
}

#[test]
fn walk_on_deep_water_is_impassable() {
    let cost = movement_cost(
        4,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!(cost > 900.0, "deep water should be impassable for walk");
}

#[test]
fn fly_ignores_terrain_cost() {
    for terrain_id in 0..=7 {
        let cost = movement_cost(
            terrain_id,
            MovementType::Fly,
            GridPos::new(0, 0),
            GridPos::new(1, 0),
        );
        assert!(
            (cost - 1.0).abs() < f32::EPSILON,
            "fly cost on terrain {} should be 1.0",
            terrain_id
        );
    }
}

#[test]
fn teleport_has_zero_cost() {
    for terrain_id in 0..=7 {
        let cost = movement_cost(
            terrain_id,
            MovementType::Teleport,
            GridPos::new(0, 0),
            GridPos::new(1, 0),
        );
        assert!(
            (cost).abs() < f32::EPSILON,
            "teleport cost on terrain {} should be 0.0",
            terrain_id
        );
    }
}

#[test]
fn swim_efficient_in_water_terrain() {
    let swamp = movement_cost(
        3,
        MovementType::Swim,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    let deep = movement_cost(
        4,
        MovementType::Swim,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    let shallow = movement_cost(
        6,
        MovementType::Swim,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );

    assert!((swamp - 1.0).abs() < f32::EPSILON);
    assert!((deep - 1.0).abs() < f32::EPSILON);
    assert!((shallow - 0.75).abs() < f32::EPSILON);
}

#[test]
fn swim_inefficient_on_land() {
    let cost = movement_cost(
        0,
        MovementType::Swim,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!(
        (cost - 2.0).abs() < f32::EPSILON,
        "swim on land should cost 2.0"
    );
}

#[test]
fn climb_efficient_on_rough_terrain() {
    let rubble = movement_cost(
        5,
        MovementType::Climb,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    let steep = movement_cost(
        7,
        MovementType::Climb,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!((rubble - 1.0).abs() < f32::EPSILON);
    assert!((steep - 1.0).abs() < f32::EPSILON);
}

#[test]
fn climb_inefficient_on_flat_terrain() {
    let cost = movement_cost(
        0,
        MovementType::Climb,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!(
        (cost - 2.0).abs() < f32::EPSILON,
        "climb on flat should cost 2.0"
    );
}

#[test]
fn unknown_terrain_defaults_to_plain() {
    let cost = movement_cost(
        99,
        MovementType::Walk,
        GridPos::new(0, 0),
        GridPos::new(1, 0),
    );
    assert!(
        (cost - 1.0).abs() < f32::EPSILON,
        "unknown terrain should default to 1.0"
    );
}

// ── path_total_cost tests ──

#[test]
fn path_total_cost_single_step_returns_zero() {
    let path = vec![GridPos::new(0, 0)];
    let cost = path_total_cost(&path, &[], MovementType::Walk);
    assert!((cost).abs() < f32::EPSILON);
}

#[test]
fn path_total_cost_two_steps_plain_terrain() {
    let path = vec![GridPos::new(0, 0), GridPos::new(1, 0)];
    let cost = path_total_cost(&path, &[0], MovementType::Walk);
    assert!((cost - 1.0).abs() < f32::EPSILON);
}

#[test]
fn path_total_cost_mixed_terrain() {
    let path = vec![GridPos::new(0, 0), GridPos::new(1, 0), GridPos::new(2, 0)];
    // step 0→1 terrain 0 (plain=1.0), step 1→2 terrain 2 (forest=2.0)
    let cost = path_total_cost(&path, &[0, 2], MovementType::Walk);
    assert!((cost - 3.0).abs() < f32::EPSILON);
}

#[test]
fn path_total_cost_without_terrain_data_defaults_to_plain() {
    let path = vec![GridPos::new(0, 0), GridPos::new(1, 0)];
    let cost = path_total_cost(&path, &[], MovementType::Walk);
    assert!(
        (cost - 1.0).abs() < f32::EPSILON,
        "missing terrain data should default to plain"
    );
}
