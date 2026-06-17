use crate::core::domains::tactical::components::GridPos;

#[test]
fn grid_pos_new_creates_zero_layer() {
    let pos = GridPos::new(3, 5);
    assert_eq!(pos.x, 3);
    assert_eq!(pos.y, 5);
    assert_eq!(pos.layer, 0);
}

#[test]
fn grid_pos_with_layer_sets_correct_layer() {
    let pos = GridPos::with_layer(10, 20, 2);
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
    assert_eq!(pos.layer, 2);
}

#[test]
fn grid_pos_manhattan_distance_computes_correctly() {
    let a = GridPos::new(0, 0);
    let b = GridPos::new(3, 4);
    assert_eq!(a.manhattan_distance(b), 7);

    let c = GridPos::new(-2, 5);
    let d = GridPos::new(3, 1);
    assert_eq!(c.manhattan_distance(d), 9);
}

#[test]
fn grid_pos_manhattan_distance_self_is_zero() {
    let pos = GridPos::new(5, 5);
    assert_eq!(pos.manhattan_distance(pos), 0);
}

#[test]
fn grid_pos_chebyshev_distance_computes_correctly() {
    let a = GridPos::new(0, 0);
    let b = GridPos::new(3, 4);
    assert_eq!(a.chebyshev_distance(b), 4); // max(3, 4)

    let c = GridPos::new(1, 2);
    let d = GridPos::new(5, 6);
    assert_eq!(c.chebyshev_distance(d), 4); // max(4, 4)
}

#[test]
fn grid_pos_chebyshev_distance_self_is_zero() {
    let pos = GridPos::new(-3, 7);
    assert_eq!(pos.chebyshev_distance(pos), 0);
}

#[test]
fn grid_pos_hex_distance_computes_correctly() {
    // (|dx| + |dy| + |dx - dy|) / 2
    let a = GridPos::new(0, 0);
    let b = GridPos::new(2, 1);
    // dx=2, dy=1: (2 + 1 + 1) / 2 = 2
    assert_eq!(a.hex_distance(b), 2);

    let c = GridPos::new(1, 2);
    let d = GridPos::new(4, 3);
    // dx=3, dy=1: (3 + 1 + 2) / 2 = 3
    assert_eq!(c.hex_distance(d), 3);
}

#[test]
fn grid_pos_hex_distance_self_is_zero() {
    let pos = GridPos::new(5, 5);
    assert_eq!(pos.hex_distance(pos), 0);
}

#[test]
fn grid_pos_neighbors_4_returns_correct_positions() {
    let center = GridPos::new(2, 2);
    let neighbors = center.neighbors_4();

    assert_eq!(neighbors.len(), 4);
    assert!(neighbors.contains(&GridPos::new(2, 1))); // up
    assert!(neighbors.contains(&GridPos::new(2, 3))); // down
    assert!(neighbors.contains(&GridPos::new(1, 2))); // left
    assert!(neighbors.contains(&GridPos::new(3, 2))); // right
}

#[test]
fn grid_pos_neighbors_4_maintains_layer() {
    let center = GridPos::with_layer(0, 0, 1);
    for neighbor in center.neighbors_4() {
        assert_eq!(neighbor.layer, 1, "neighbor layer should match center");
    }
}

#[test]
fn grid_pos_neighbors_8_returns_eight_positions() {
    let center = GridPos::new(0, 0);
    let neighbors = center.neighbors_8();

    assert_eq!(neighbors.len(), 8);
    // All surrounding positions
    assert!(neighbors.contains(&GridPos::new(-1, -1)));
    assert!(neighbors.contains(&GridPos::new(0, -1)));
    assert!(neighbors.contains(&GridPos::new(1, -1)));
    assert!(neighbors.contains(&GridPos::new(-1, 0)));
    assert!(neighbors.contains(&GridPos::new(1, 0)));
    assert!(neighbors.contains(&GridPos::new(-1, 1)));
    assert!(neighbors.contains(&GridPos::new(0, 1)));
    assert!(neighbors.contains(&GridPos::new(1, 1)));
}

#[test]
fn grid_pos_equality() {
    assert_eq!(GridPos::new(1, 2), GridPos::new(1, 2));
    assert_ne!(GridPos::new(1, 2), GridPos::new(2, 1));
    assert_ne!(GridPos::with_layer(1, 2, 0), GridPos::with_layer(1, 2, 1));
}

#[test]
fn grid_pos_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(GridPos::new(3, 4));
    set.insert(GridPos::new(3, 4)); // duplicate
    assert_eq!(set.len(), 1);
}
