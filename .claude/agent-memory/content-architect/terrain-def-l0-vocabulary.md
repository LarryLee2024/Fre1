---
name: terrain-def-l0-vocabulary
description: TerrainDef is an L0 Vocabulary Def defining fundamental terrain types, not an L4 world detail
metadata:
  type: project
---

TerrainDef belongs to L0 Vocabulary, not L4 World, because:
1. Terrain types (plain, forest, mountain, water) are fundamental vocabulary items like elements and damage types
2. TerrainDef is consumed by L1 (movement costs via GridMap), L2 (entity placement checks), L3 (combat bonuses), and L4 (map rendering) — the multi-layer consumption pattern is the defining characteristic of L0
3. TerrainDef does not reference any other Def (satisfies L0 self-containment rule)
4. The old prototype's `char_code` field was removed because TMX uses GID, not char mapping
5. TerrainDef uses `Concealment` and `TerrainFlags` as inline enums/structs (same pattern as FactionDef's `FactionAttitude`)

Implication: `assets/config/00_vocabulary/terrains.ron` is the correct directory, not an L4 directory. The `terrain_schema.md` in docs/04-data/ is listed under L4 in the content-layering doc but this refers to the *data schema* layer (runtime structures like SurfaceType, HazardZoneDef), not the Content Def layer.

**Why:** Separates "what terrains exist" (L0 vocabulary, stable) from "how terrain data flows at runtime" (L4 schema, may change with engine updates).

**How to apply:** When placing new Def types, check whether the type is consumed by multiple layers above it (L0) or specific to one layer's implementation (L4).
