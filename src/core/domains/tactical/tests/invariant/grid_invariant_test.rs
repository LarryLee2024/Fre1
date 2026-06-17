use crate::core::domains::tactical::components::GridPos;
use crate::core::domains::tactical::resources::{GridLayout, GridMap, TileData, TileFlags};

/// 不变量：网格中每个 Tile 的 terrain_def_id + height + flags 都能通过 packed u32 无损还原。
#[test]
fn tile_data_packing_is_lossless() {
    for terrain_id in [0, 1, 255, u16::MAX] {
        for height in [0, 1, 128, u8::MAX] {
            for flags in [
                TileFlags(0),
                TileFlags::PASSABLE,
                TileFlags(0b1010_1010),
                TileFlags(u8::MAX),
            ] {
                let data = TileData::new(terrain_id, height, flags);
                assert_eq!(
                    data.terrain_def_id(),
                    terrain_id,
                    "terrain_id mismatch: packed={}, original={}",
                    data.terrain_def_id(),
                    terrain_id
                );
                assert_eq!(
                    data.height(),
                    height,
                    "height mismatch: packed={}, original={}",
                    data.height(),
                    height
                );
                assert_eq!(
                    data.flags(),
                    flags,
                    "flags mismatch: packed={:?}, original={:?}",
                    data.flags(),
                    flags
                );
            }
        }
    }
}

/// 不变量：GridMap 的 tile 数量始终等于 width * height。
#[test]
fn grid_tile_count_matches_dimensions() {
    let cases = [(1, 1), (10, 10), (20, 15), (100, 1), (1, 100)];
    for (w, h) in &cases {
        let map = GridMap::new(*w, *h, GridLayout::Square);
        assert_eq!(
            map.tiles.len(),
            (*w * *h) as usize,
            "tile count {} != width {} * height {}",
            map.tiles.len(),
            w,
            h
        );
    }
}

/// 不变量：in_bounds 在合法坐标上始终返回 true。
#[test]
fn grid_in_bounds_all_valid_positions() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    for x in 0..10 {
        for y in 0..10 {
            assert!(
                map.in_bounds(GridPos::new(x, y)),
                "position ({}, {}) should be in bounds",
                x,
                y
            );
        }
    }
}

/// 不变量：in_bounds 在非法坐标上始终返回 false。
#[test]
fn grid_in_bounds_all_invalid_positions() {
    let map = GridMap::new(10, 10, GridLayout::Square);
    let invalid = [
        GridPos::new(-1, 0),
        GridPos::new(0, -1),
        GridPos::new(10, 0),
        GridPos::new(0, 10),
        GridPos::new(-5, -5),
        GridPos::new(100, 100),
    ];
    for pos in &invalid {
        assert!(
            !map.in_bounds(*pos),
            "position {:?} should be out of bounds",
            pos
        );
    }
}

/// 不变量：get_tile 返回的引用与 get_tile_mut 返回的内容一致。
#[test]
fn grid_get_tile_immutable_mut_consistent() {
    let mut map = GridMap::new(5, 5, GridLayout::Square);
    let pos = GridPos::new(2, 3);

    if let Some(tile) = map.get_tile_mut(pos) {
        *tile = TileData::new(7, 3, TileFlags::PASSABLE);
    }

    let read = map.get_tile(pos);
    assert!(read.is_some());
    assert_eq!(read.unwrap().terrain_def_id(), 7);
    assert_eq!(read.unwrap().height(), 3);
    assert!(read.unwrap().is_passable());
}

/// 不变量：tiles_in_range 从不返回超出网格边界的坐标。
#[test]
fn tiles_in_range_never_returns_out_of_bounds() {
    let map = GridMap::new(7, 7, GridLayout::Square);
    let centers = [
        GridPos::new(0, 0),
        GridPos::new(3, 3),
        GridPos::new(6, 6),
        GridPos::new(0, 6),
    ];

    for center in &centers {
        let reachable = map.tiles_in_range(*center, 5);
        for pos in &reachable {
            assert!(
                map.in_bounds(*pos),
                "tiles_in_range from {:?} returned out-of-bounds {:?}",
                center,
                pos
            );
        }
    }
}

/// 不变量：tiles_in_range 返回的所有位置与起点的曼哈顿距离 ≤ range。
#[test]
fn tiles_in_range_respects_max_distance() {
    let map = GridMap::new(20, 20, GridLayout::Square);
    let center = GridPos::new(10, 10);

    for range in [1, 2, 3, 5, 10] {
        let reachable = map.tiles_in_range(center, range);
        for pos in &reachable {
            let dist = center.manhattan_distance(*pos);
            assert!(
                dist <= range,
                "position {:?} at distance {} exceeds range {}",
                pos,
                dist,
                range
            );
        }
    }
}

/// 不变量：neighbors_4 从不返回超出边界的坐标。
#[test]
fn neighbors_4_never_returns_out_of_bounds() {
    let map = GridMap::new(5, 5, GridLayout::Square);
    for x in 0..5 {
        for y in 0..5 {
            let neighbors = map.neighbors_4(GridPos::new(x, y));
            for n in &neighbors {
                assert!(
                    map.in_bounds(*n),
                    "neighbors_4 of ({}, {}) returned out-of-bounds {:?}",
                    x,
                    y,
                    n
                );
            }
        }
    }
}
