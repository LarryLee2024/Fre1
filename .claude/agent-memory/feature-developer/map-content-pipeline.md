---
name: map-content-pipeline
description: Map content pipeline module structure at src/infra/map/ with types, asset, loader, renderer, systems
metadata:
  type: reference
---

## Map Content Pipeline (src/infra/map/)

### Module Structure

- **types.rs** — MapObjectGuid(u64), PropertyMap, PropertyValue, ObjectShape, MapGridLayout, MapHexDirection, MapTileFlags(u8). Independent serde-friendly types for the Asset layer.
- **asset.rs** — MapAsset (Asset + TypePath) with MapMetadata, TerrainGrid, TileEntry, ObjectLayer, MapObject, MapObjectPos, SpawnPoint, MapRegion, NavigationMask. Immutable, no game logic. Events are NOT here (they're in events.rs).
- **events.rs** — MapLoadedEvent { map_asset_id }, MapUnloadedEvent { map_asset_id }. Minimal event types.
- **importer.rs** — Build-time TMX→MapAsset conversion functions: generate_guid(), pixel_to_gridpos(), build_navigation_mask(), validate_*(). Not needed at runtime.
- **loader.rs** — Runtime MapAsset→GridMap conversion: TerrainIndex Resource, build_terrain_index(), convert_to_gridmap(), convert_grid_layout(), convert_tile_flags(), convert_hex_direction(), tile_world_position().
- **renderer/spawn.rs** — V1 Entity-per-Tile: MapRoot, MapTileMarker, TileVisual components. spawn_tile_entities() pure function. TextureAtlas rendering is TODO.
- **renderer/overlay.rs** — Overlay components: OverlayRoot, CursorHighlight, MovementRangeOverlay, InteractionOverlay, OverlayCell, OverlayType.
- **renderer/cleanup.rs** — Entity cleanup: despawn_map_entities(), despawn_overlay_entities().
- **systems/map_loader_system.rs** — load_map_into_world() function (not a Bevy system yet, called directly).
- **systems/map_cleanup_system.rs** — cleanup_map_world(), cleanup_overlay() functions.
- **systems/object_instantiator.rs** — ObjectMarker, SpawnPointMarker, DecorationMarker, InteractiveMarker, TriggerZoneMarker. instantiate_object(), instantiate_spawn_point(), instantiate_object_layer().
- **plugin.rs** — MapPlugin registers MapAsset, TerrainIndex, MapRenderConfig.
- **mod.rs** — Module header re-exporting public API.

### Key Design Decisions

- MapAsset uses String terrain_id (not u16) — RON-readable. u16 conversion happens in loader.rs via TerrainIndex.
- MapObjectPos is a simplified {x, y} to avoid direct dependency on tactical domain's GridPos in the Asset layer.
- MapTileFlags is independent from tactical domain's TileFlags — same bit layout but separate types, converted via convert_tile_flags().
- V1 renderer avoids SpriteSheetBundle/TextureAtlas (removed in Bevy 0.19). TODO for V2 with Material2D batch.
- In Bevy 0.19: set_parent → set_parent_in_place, despawn_recursive → despawn, SpatialBundle removed (use Transform directly), add_event is automatic via #[derive(Event)].

### Content Integration

- TerrainDef (at src/content/terrain_def.rs) with TerrainFlags/Concealment.
- LoadedTerrainDefs resource added to content_plugin.rs.
- load_terrain_content() is a standalone Startup system (separate from load_all_content to avoid Bevy's 16-param system limit).
- 4 terrain RON files: plain.ron, forest.ron, mountain.ron, water.ron.
- MapPlugin registered in Phase 8 after CameraPlugin.
