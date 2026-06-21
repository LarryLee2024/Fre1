---
name: map-def-mapasset-dual-identity
description: MapDef (content document) is the same concept as MapAsset (Bevy Asset) — dual identity for the L4 world data
metadata:
  type: project
---

MapDef (content definition document) and MapAsset (Bevy Asset struct) are the same concept viewed from different angles:
- MapDef: the L4 Content Def type discussed in architecture/design docs
- MapAsset: the `#[derive(Asset, TypePath)]` Rust struct in `src/infra/map/asset.rs`

Unlike other Defs (TagDef, FactionDef, etc.), MapAsset does NOT use `DefRegistry<T>` because:
1. Map data is too large for HashMap-style registry (100x100 grids + object layers)
2. Access pattern is position-based (GridPos → TileData), not ID-based
3. Lifecycle is scene-scoped (load on OnEnter, unload on OnExit), not application-global
4. Data source is Importer-generated TMX output, not hand-written RON

This means the Content Pipeline's standard Load → Deserialize → Validate → Register → Freeze sequence applies differently: Register is replaced by AssetServer management + GridMap construction.

**Why:** ADR-065 design decision — maps are too large and access-pattern different to fit the standard DefRegistry pattern.

**How to apply:** This dual-identity pattern only applies to MapDef. Other L4 Defs (SceneDef, CutsceneDef) should follow the standard DefRegistry pattern since they are smaller and accessed by ID.
