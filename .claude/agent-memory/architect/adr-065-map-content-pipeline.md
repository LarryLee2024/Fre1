---
name: adr-065-map-content-pipeline
description: ADR-065 defines the Map Content Pipeline — Tiled TMX to runtime MapAsset pipeline with Tile-Config separation, Object Layer as first-class citizen, and custom MapRenderer.
metadata:
  type: reference
---

# ADR-065 Map Content Pipeline Architecture

File: `docs/01-architecture/40-cross-cutting/ADR-065-map-content-pipeline.md`
Status: Proposed (2026-06-22)

## Core Decisions

1. **Three-layer pipeline**: Tiled (TMX, editing format) → Importer (build-time tool in `tools/map_importer/`) → MapAsset (RON, runtime format). Game binary never parses TMX.

2. **Tile-Config separation**: Tile stores only `terrain_id: TerrainDefId`. All gameplay values (move_cost, defense_bonus) come from TerrainDef in Config Registry. See [[definition-instance-separation]].

3. **Object Layer as first-class citizen**: Objects in MapAsset have stable GUIDs (content-hash based, deterministic). Object `class` field drives instantiation at runtime via domain systems (SpawnPoint→SpawnSystem, Chest→InteractionSystem, etc.). Objects are definitions, not entities.

4. **Custom MapRenderer** in `src/infra/map/renderer/` — no bevy_ecs_tilemap. Uses Material2d batch rendering for tiles, separate overlay passes for highlights (movement range, AOE, cursor).

5. **Scene lifecycle**: Map loaded on `OnEnter(TacticalMap/Combat)` via ADR-050 ScenePlugin. MapLoader creates GridMap Resource, instantiates object entities as SceneRoot children. Cleaned up on `OnExit`.

6. **MapAsset structure**: metadata + terrain_grid + object_layers + spawn_points + regions + navigation_mask (pre-computed passability).

7. **Importer validation**: GUID uniqueness, TerrainId validity, reference integrity, height continuity, grid completeness.

## Forbidden Highlights
- Tile must NOT store gameplay values
- Game binary must NOT parse TMX
- Object must NOT directly map to Entity
- MapRenderer must NOT query domain components
- MapAsset must NOT be mutated at runtime

## Out of Scope (v1)
World Map, Fog of War, Dynamic Map, Runtime Editing, Chunk Streaming, Region runtime queries.

## Next Steps (in order)
1. @content-architect: MapDef + TerrainDef content definitions
2. @data-architect: MapAsset Schema + Importer Schema
3. @presentation-architect: MapRenderer architecture
4. @feature-developer: Importer + Renderer implementation

## Related Documents
- ADR-022 (grid/terrain/faction) — this ADR supplements, doesn't replace
- ADR-047 (content loading pipeline) — for Asset registration pattern
- ADR-050 (game state machine) — for scene lifecycle pattern
- docs/09-planning/map-content-pipeline-plan.md — action plan
