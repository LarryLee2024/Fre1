---
name: map-asset-schema
description: MapAsset 四层架构设计（Definition 层特例）、ID 语义化例外、双 Tile 表示
metadata:
  type: reference
---

# MapAsset Schema Architecture

## Key Decisions

### MapAsset is Definition layer (special case)
MapAsset is a Bevy Asset, not a DefRegistry member. It follows Definition-layer immutability rules but uses AssetServer path loading + scene lifecycle (OnEnter/OnExit). Not stored in DefRegistry due to: large size, positional access pattern, scene-scoped lifecycle.

### Dual Tile Representation
- `TerrainGrid.tiles: Vec<TileEntry>` in MapAsset RON — string TerrainId ("ter:plain"), human-readable
- `GridMap.tiles: Vec<TileData>` in runtime — packed u32 (u16 terrain_id + u8 height + u8 flags)
- Conversion happens in MapLoader during scene OnEnter
- This keeps RON files reviewable/diffable while maintaining runtime performance

### MapDef ID Semantic Exception
MapDef uses semantic names (`map:dragon_peak`) rather than standard numeric IDs (`map_000001`). Reason: L4 World layer, AssetServer path loading, level designer naming control. Documented exception in ID strategy rules. See [[id-strategy-exceptions]].

### Importer Schema
TMX → MapAsset mapping defined in `map-importer-schema.md`. Includes:
- GID→TerrainId via tileset_mappings.toml (not TMX terrain attribute)
- Content-hash GUID (SipHash-2-4 with fixed keys for determinism)
- Pre-computed NavigationMask from TerrainDef.flags
- 8 validation rules (5 error, 3 warning)

### Schema file locations
- `docs/04-data/infrastructure/map-asset-schema.md` — MapAsset runtime schema
- `docs/04-data/infrastructure/map-importer-schema.md` — TMX→MapAsset mapping spec
