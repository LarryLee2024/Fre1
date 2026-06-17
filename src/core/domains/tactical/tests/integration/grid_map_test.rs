use crate::core::domains::tactical::components::GridPos;
use crate::core::domains::tactical::resources::{GridLayout, GridMap, TileData, TileFlags};

#[test]
fn grid_map_new_creates_correct_sized_grid() {
    let map = GridMap::new(10, 15, GridLayout::Square);
    assert_eq!(map.width, 10);
    assert_eq!(map.height, 15);
    assert_eq!(map.tiles.len(), 150);
}

#[test]
fn grid_map_new_all_tiles_passable_by_default() {
    let map = GridMap::new(5, 5, GridLayout::Square);
    for tile in &map.tiles {
        assert!(tile.is_passable(), "default tile should be passable");
    }
}

#[test]
fn grid_map_from_tiles_validates_size() {
    let tiles = vec![TileData::new(1, 0, TileFlags::PASSABLE); 6];
    let map = GridMap::from_tiles(2, 3, tiles, GridLayout::Square);
    assert_eq!(map.width, 2);
    assert_eq!(map.height, 3);
    assert_eq!(map.tiles.len(), 6);
}

#[test]
#[should_panic(expected = "tile count must match")]
fn grid_map_from_tiles_panics_on_size_mismatch() {
    let tiles = vec![TileData::new(1, 0, TileFlags::PASSABLE); 5]; // 5 != 2*3
    GridMap::from_tiles(2, 3, tiles, GridLayout::Square);
}

#[test]
fn grid_map_in_bounds_positive() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    assert!(map.in_bounds(GridPos::new(0, 0)));
    assert!(map.in_bounds(GridPos::new(9, 9)));
    assert!(map.in_bounds(GridPos::new(5, 5)));
}

#[test]
fn grid_map_in_bounds_negative() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    assert!(
        !map.in_bounds(GridPos::new(-1, 0)),
        "negative x should be out of bounds"
    );
    assert!(
        !map.in_bounds(GridPos::new(0, -1)),
        "negative y should be out of bounds"
    );
    assert!(
        !map.in_bounds(GridPos::new(10, 0)),
        "x >= width should be out of bounds"
    );
    assert!(
        !map.in_bounds(GridPos::new(0, 10)),
        "y >= height should be out of bounds"
    );
}

#[test]
fn grid_map_get_tile_returns_tile_at_position() {
    let mut map = GridMap::new(5, 5, GridLayout::Square);

    // Set a specific tile
    if let Some(tile) = map.get_tile_mut(GridPos::new(2, 3)) {
        *tile = TileData::new(42, 5, TileFlags(0));
    }

    let tile = map.get_tile(GridPos::new(2, 3));
    assert!(tile.is_some());
    assert_eq!(tile.unwrap().terrain_def_id(), 42);
    assert_eq!(tile.unwrap().height(), 5);
}

#[test]
fn grid_map_get_tile_out_of_bounds_returns_none() {
    let map = GridMap::new(5, 5, GridLayout::Square);
    assert!(map.get_tile(GridPos::new(10, 10)).is_none());
    assert!(map.get_tile(GridPos::new(-1, 0)).is_none());
}

#[test]
fn grid_map_neighbors_4_filters_out_of_bounds() {
    let map = GridMap::new(3, 3, GridLayout::Square);
    // Corner (0,0) should have only 2 neighbors
    let neighbors = map.neighbors_4(GridPos::new(0, 0));
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&GridPos::new(0, 1)));
    assert!(neighbors.contains(&GridPos::new(1, 0)));

    // Center (1,1) should have 4 neighbors
    let center_neighbors = map.neighbors_4(GridPos::new(1, 1));
    assert_eq!(center_neighbors.len(), 4);
}

#[test]
fn grid_map_tiles_in_range_respects_boundary() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    let reachable = map.tiles_in_range(GridPos::new(0, 0), 3);
    for pos in &reachable {
        assert!(
            map.in_bounds(*pos),
            "reachable position {:?} should be in bounds",
            pos
        );
        let dist = GridPos::new(0, 0).manhattan_distance(*pos);
        assert!(
            dist <= 3,
            "position {:?} at distance {} exceeds range",
            pos,
            dist
        );
    }
}

#[test]
fn grid_map_tiles_in_range_excludes_start() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    let reachable = map.tiles_in_range(GridPos::new(5, 5), 5);
    assert!(
        !reachable.contains(&GridPos::new(5, 5)),
        "center should not be in results"
    );
}

#[test]
fn grid_map_tiles_in_range_skips_impassable_tiles() {
    let mut map = GridMap::new(5, 5, GridLayout::Square);
    // Block the tile at (1,0)
    if let Some(tile) = map.get_tile_mut(GridPos::new(1, 0)) {
        *tile = TileData::new(0, 0, TileFlags(0)); // not passable
    }

    let reachable = map.tiles_in_range(GridPos::new(0, 0), 3);
    assert!(
        !reachable.contains(&GridPos::new(1, 0)),
        "blocked tile should not be reachable"
    );
    // (2,0) should also not be reachable since it requires going through (1,0)
    assert!(
        !reachable.contains(&GridPos::new(2, 0)),
        "position past blocked tile should not be reachable"
    );
}

#[test]
fn grid_map_tiles_in_range_out_of_bounds_center() {
    let map = GridMap::new(5, 5, GridLayout::Square);
    let reachable = map.tiles_in_range(GridPos::new(10, 10), 3);
    assert!(
        reachable.is_empty(),
        "out of bounds center should return empty"
    );
}

#[test]
fn grid_map_grid_to_world_square() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    let (wx, wy) = map.grid_to_world(GridPos::new(3, 4));
    assert!((wx - 3.0).abs() < f32::EPSILON);
    assert!((wy - 4.0).abs() < f32::EPSILON);
}

#[test]
fn grid_map_world_to_grid_square() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    let pos = map.world_to_grid(3.2, 4.7);
    assert_eq!(pos, Some(GridPos::new(3, 5)));

    let out = map.world_to_grid(15.0, 15.0);
    assert_eq!(out, None, "out of bounds world coords should return None");
}

#[test]
fn grid_map_default_layout_is_square() {
    let map = GridMap::new(20, 15, GridLayout::Square);
    assert_eq!(map.layout, GridLayout::Square);
}
