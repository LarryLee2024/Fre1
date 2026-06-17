use crate::core::domains::tactical::resources::{TileData, TileFlags};

// ── TileFlags ──

#[test]
fn tile_flags_passable_contains_flag() {
    let flags = TileFlags::PASSABLE;
    assert!(flags.is_passable());
}

#[test]
fn tile_flags_empty_not_passable() {
    let flags = TileFlags(0);
    assert!(!flags.is_passable());
}

#[test]
fn tile_flags_contains_multiple_flags() {
    let flags = TileFlags(0b0000_0101); // PASSABLE + BUILDABLE
    assert!(flags.contains(TileFlags::PASSABLE));
    assert!(flags.contains(TileFlags::BUILDABLE));
    assert!(!flags.contains(TileFlags::BLOCKS_SIGHT));
    assert!(!flags.contains(TileFlags::FLYABLE));
}

#[test]
fn tile_flags_contains_exact() {
    let flags = TileFlags::PASSABLE;
    assert!(flags.contains(TileFlags::PASSABLE));
}

#[test]
fn tile_flags_combined_all() {
    let all = TileFlags(0b0000_1111);
    assert!(all.contains(TileFlags::PASSABLE));
    assert!(all.contains(TileFlags::FLYABLE));
    assert!(all.contains(TileFlags::BUILDABLE));
    assert!(all.contains(TileFlags::BLOCKS_SIGHT));
}

// ── TileData ──

#[test]
fn tile_data_round_trips_packed_values() {
    let data = TileData::new(42, 7, TileFlags::PASSABLE);
    assert_eq!(data.terrain_def_id(), 42);
    assert_eq!(data.height(), 7);
    assert_eq!(data.flags(), TileFlags::PASSABLE);
}

#[test]
fn tile_data_zero_initialized() {
    let data = TileData::new(0, 0, TileFlags(0));
    assert_eq!(data.terrain_def_id(), 0);
    assert_eq!(data.height(), 0);
    assert_eq!(data.flags(), TileFlags(0));
}

#[test]
fn tile_data_max_values() {
    let data = TileData::new(u16::MAX, u8::MAX, TileFlags(u8::MAX));
    assert_eq!(data.terrain_def_id(), u16::MAX);
    assert_eq!(data.height(), u8::MAX);
    assert_eq!(data.flags(), TileFlags(u8::MAX));
}

#[test]
fn tile_data_is_passable_when_flag_set() {
    let passable = TileData::new(1, 0, TileFlags::PASSABLE);
    assert!(passable.is_passable());

    let blocked = TileData::new(1, 0, TileFlags(0));
    assert!(!blocked.is_passable());
}

#[test]
fn tile_data_clone_equality() {
    let a = TileData::new(10, 3, TileFlags::PASSABLE);
    let b = TileData::new(10, 3, TileFlags::PASSABLE);
    assert_eq!(a, b);

    let c = TileData::new(10, 3, TileFlags(0));
    assert_ne!(a, c, "different flags should not be equal");
}
