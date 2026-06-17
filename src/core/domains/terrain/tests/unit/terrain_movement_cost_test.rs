use crate::core::domains::terrain::components::TerrainType;
use crate::core::domains::terrain::rules::movement_cost::{MoveCategory, terrain_movement_cost};

#[test]
fn walk_on_normal_uses_one_mp() {
    assert_eq!(
        terrain_movement_cost(TerrainType::Normal, MoveCategory::Walk),
        1.0
    );
}

#[test]
fn walk_on_obstacle_is_blocked() {
    assert_eq!(
        terrain_movement_cost(TerrainType::Obstacle, MoveCategory::Walk),
        f32::MAX
    );
}

#[test]
fn fly_ignores_terrain() {
    for t in TerrainType::ALL {
        assert_eq!(terrain_movement_cost(*t, MoveCategory::Fly), 1.0);
    }
}

#[test]
fn swim_in_water_is_efficient() {
    assert_eq!(
        terrain_movement_cost(TerrainType::Water, MoveCategory::Swim),
        1.0
    );
    assert_eq!(
        terrain_movement_cost(TerrainType::Water, MoveCategory::Walk),
        2.0
    );
}

#[test]
fn teleport_cost_zero() {
    for t in TerrainType::ALL {
        assert_eq!(terrain_movement_cost(*t, MoveCategory::Teleport), 0.0);
    }
}
