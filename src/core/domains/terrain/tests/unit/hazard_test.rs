use crate::core::domains::terrain::components::{
    Concealment, Passability, SurfaceType, TerrainType, TileProperties,
};
use crate::core::domains::terrain::resources::{
    HazardTriggerCondition, HazardVisibility, HazardZoneDef,
};

fn make_hazard(id: &str) -> HazardZoneDef {
    HazardZoneDef {
        id: id.to_string(),
        trigger_condition: HazardTriggerCondition::OnEnter,
        effects: vec!["eff_000001".to_string()],
        is_consumable: false,
        visibility: HazardVisibility::Visible,
    }
}

fn make_tile_props(surface: SurfaceType) -> TileProperties {
    TileProperties::new(
        TerrainType::Normal,
        0,
        Passability::Walkable,
        Concealment::None,
        surface,
    )
}

#[test]
fn matches_tile_returns_false_for_any_surface() {
    let hazard = make_hazard("haz_001");
    let surfaces = [
        SurfaceType::Normal,
        SurfaceType::Poison,
        SurfaceType::Burning,
        SurfaceType::Lava,
        SurfaceType::Ice,
        SurfaceType::Water,
        SurfaceType::Oil,
    ];
    for surface in &surfaces {
        let props = make_tile_props(*surface);
        assert!(
            !hazard.matches_tile(&props),
            "matches_tile should return false for surface {:?} (safety default)",
            surface
        );
    }
}

#[test]
fn matches_tile_returns_false_for_empty_hazard() {
    let hazard = make_hazard("haz_002");
    let props = make_tile_props(SurfaceType::Normal);
    assert!(!hazard.matches_tile(&props));
}
