---
name: map-renderer-architecture
description: MapRenderer architecture doc completed at docs/06-ui/04-data-flow/map-rendering.md with 18 sections covering 6-layer rendering, Material2d batching, overlay system, coordinate alignment with GridMap
metadata:
  type: project
---

MapRenderer architecture delivered at `/Users/lf380/Code/Bevy/Fre/docs/06-ui/04-data-flow/map-rendering.md`.

**Key architecture decisions documented:**
- Six-layer rendering: Terrain (z=0.0) -> Decoration -> Grid Lines -> Overlay -> Cursor -> Unit (z=3.0), with `map_z` constants module for z-value management
- Material2d batch processing for terrain tiles (not Entity-per-Tile), with TextureAtlas mapping terrain_id to UV coordinates
- Overlay layer: 5 independent Mesh+Material entities (Movement, AOE, Threat, Path, Hover) each with dirty-flag update pattern -- not vertex-color encoding
- Cursor layer as independent Sprite Entity (frequent updates, not worth batch rebuild)
- Coordinate system: GridMap.grid_to_world() is the single source of truth for GridPos->WorldPos, shared between MapRenderer and Domain
- Unit rendering: on unit Entity's own Sprite+Transform, not inside MapRenderer; sync_unit_position bridge system reads GridPos -> writes Transform
- MapRenderer in infra layer (L2, `src/infra/map/renderer/`), uses CameraQuery, camera does not depend on MapRenderer
- MapOverlayData as the bridge Resource: Domain writes overlay sets -> App bridge -> MapOverlayData -> MapRenderer consumes
- Scene lifecycle: OnEnter(Combat) builds terrain batch + overlays + cursor; OnExit cleans up; after( map_loader_system ) explicit ordering
- V1 fast path: Entity-per-Tile with TextureAtlas; V2 target: Material2d batch; both described as phased implementation

**Layered interaction with existing architecture:**
- References camera-ui-interaction.md for screen_to_world / world_to_screen conversion
- References ADR-065 for asset pipeline and MapAsset structure
- Data flow pattern: Domain writes overlay sets -> App bridge -> MapOverlayData (infra Resource) -> MapRenderer consumes
- NOT in UI layer -- uses world-space 2D rendering (Sprite/Mesh2d), not UI Node system

**README updates:**
- Added map-rendering.md to directory tree index in docs/06-ui/README.md
- Added upstream mapping row for ADR-065 SS7
- Added MapRenderer row to architecture layer overview table
