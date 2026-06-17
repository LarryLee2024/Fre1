use crate::core::domains::tactical::components::GridPos;
use crate::core::domains::tactical::rules::range::{attack_range, bfs_reachable_positions};

#[test]
fn bfs_reachable_positions_empty_when_center_is_blocked() {
    let center = GridPos::new(0, 0);
    let reachable = bfs_reachable_positions(
        center,
        5.0,
        &|_| true,
        &|_| false, // nothing passable
        &|_, _| 1.0,
    );
    assert!(
        reachable.is_empty(),
        "no positions should be reachable when center is blocked"
    );
}

#[test]
fn bfs_reachable_positions_respects_max_cost() {
    let center = GridPos::new(0, 0);
    let reachable = bfs_reachable_positions(
        center,
        2.0,
        &|p| p.x.abs() <= 5 && p.y.abs() <= 5,
        &|_| true,
        &|_, _| 1.0,
    );
    // All positions within Manhattan distance 2, excluding center
    for pos in &reachable {
        let dist = center.manhattan_distance(*pos);
        assert!(
            dist <= 2,
            "position {:?} at distance {} exceeds max_cost 2",
            pos,
            dist
        );
    }
    assert!(
        !reachable.contains(&center),
        "center should not be in reachable set"
    );
}

#[test]
fn bfs_reachable_positions_handles_variable_costs() {
    let center = GridPos::new(0, 0);
    let reachable = bfs_reachable_positions(
        center,
        3.0,
        &|p| p.x.abs() <= 5 && p.y.abs() <= 5,
        &|_| true,
        &|from, to| {
            if from.x == 0 && from.y == 0 && to.x == 1 && to.y == 0 {
                3.0 // expensive first step east
            } else {
                1.0
            }
        },
    );
    // The expensive path east (cost 3.0) reaches (1,0) but not beyond
    assert!(
        reachable.contains(&GridPos::new(1, 0)),
        "(1,0) should be reachable via expensive step"
    );
    assert!(
        !reachable.contains(&GridPos::new(2, 0)),
        "(2,0) beyond expensive step should not be reachable"
    );
    // Cheap paths in other directions can go further
    assert!(
        reachable.contains(&GridPos::new(0, 3)),
        "(0,3) should be reachable via cheap path (cost 1+1+1)"
    );
    // Teleport (0 MP cost) should still work since cost_fn returns 1.0 for all other paths
    assert!(
        reachable.contains(&GridPos::new(-1, 0)),
        "(-1,0) should be reachable via cheap path"
    );
}

#[test]
fn bfs_reachable_positions_out_of_bounds() {
    let center = GridPos::new(10, 10);
    let reachable = bfs_reachable_positions(
        center,
        5.0,
        &|p| p.x >= 8 && p.x <= 12 && p.y >= 8 && p.y <= 12,
        &|_| true,
        &|_, _| 1.0,
    );
    for pos in &reachable {
        assert!(
            pos.x >= 8 && pos.x <= 12,
            "position {:?} out of bounds",
            pos
        );
        assert!(
            pos.y >= 8 && pos.y <= 12,
            "position {:?} out of bounds",
            pos
        );
    }
}

#[test]
fn attack_range_minimal() {
    let center = GridPos::new(5, 5);
    let range = attack_range(center, 1, 2);
    for pos in &range {
        let dist = center.chebyshev_distance(*pos);
        assert!(
            dist >= 1 && dist <= 2,
            "position {:?} at distance {} outside [1,2]",
            pos,
            dist
        );
    }
    assert!(
        !range.contains(&center),
        "center should not be in attack range"
    );
}

#[test]
fn attack_range_returns_correct_count() {
    let center = GridPos::new(0, 0);
    // range 1-1: all 8 neighbors
    let range1 = attack_range(center, 1, 1);
    assert_eq!(range1.len(), 8, "melee range should have 8 positions");

    // range 2-2: all positions at chebyshev distance 2 (24 - 8 inner ring = 16)
    let range2 = attack_range(center, 2, 2);
    assert_eq!(range2.len(), 16, "range 2 ring should have 16 positions");
}

#[test]
fn attack_range_long_range() {
    let center = GridPos::new(0, 0);
    let range = attack_range(center, 3, 5);
    for pos in &range {
        let dist = center.chebyshev_distance(*pos);
        assert!(
            dist >= 3 && dist <= 5,
            "position {:?} at distance {} outside [3,5]",
            pos,
            dist
        );
    }
}

#[test]
fn attack_range_zero_min_range_includes_all_within_max() {
    let center = GridPos::new(0, 0);
    let range = attack_range(center, 0, 1);
    // Chebyshev distance 0-1, excluding center
    assert_eq!(range.len(), 8, "range 0-1 should have 8 neighbors");
    assert!(!range.contains(&center), "center should not be included");
}
