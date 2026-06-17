use crate::core::domains::terrain::components::{
    Concealment, Passability, SurfaceType, TerrainType, TileProperties,
};

fn make_props(terrain: TerrainType, surface: SurfaceType) -> TileProperties {
    TileProperties::new(
        terrain,
        0,
        Passability::Walkable,
        Concealment::None,
        surface,
    )
}

// ─── 不变量 3.1: 通行性一致性 ────────────────────────────

#[test]
fn passability_lava_is_impassable() {
    let props = make_props(TerrainType::Normal, SurfaceType::Lava);
    assert_eq!(props.current_passability(), Passability::Impassable);
}

#[test]
fn passability_normal_surface_returns_base() {
    let props = make_props(TerrainType::Normal, SurfaceType::Normal);
    assert_eq!(props.current_passability(), Passability::Walkable);
}

#[test]
fn passability_ice_returns_base() {
    let props = make_props(TerrainType::Normal, SurfaceType::Ice);
    assert_eq!(props.current_passability(), Passability::Walkable);
}

#[test]
fn passability_water_is_impassable() {
    let props = make_props(TerrainType::Normal, SurfaceType::Water);
    assert_eq!(props.current_passability(), Passability::Impassable);
}

// ─── 不变量 3.1: 遮蔽度一致性 ────────────────────────────

#[test]
fn concealment_bush_is_half() {
    let props = make_props(TerrainType::Bush, SurfaceType::Normal);
    assert_eq!(props.current_concealment(), Concealment::Half);
}

#[test]
fn concealment_highground_is_none() {
    let props = make_props(TerrainType::Highground, SurfaceType::Normal);
    assert_eq!(props.current_concealment(), Concealment::None);
}

#[test]
fn concealment_normal_returns_base() {
    let props = make_props(TerrainType::Normal, SurfaceType::Normal);
    assert_eq!(props.current_concealment(), Concealment::None);
}
