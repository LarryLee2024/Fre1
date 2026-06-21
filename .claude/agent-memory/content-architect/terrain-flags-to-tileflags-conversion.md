---
name: terrain-flags-to-tileflags-conversion
description: TerrainDef.flags (bool struct) maps to Tactical TileFlags (u8 bitmask) at Importer time
metadata:
  type: project
---

TerrainDef uses a bool-struct `TerrainFlags { passable, flyable, buildable, blocks_sight }` for content author readability. The conversion to `TileFlags(u8)` bitmask happens in the Map Importer, not at runtime.

This is a deliberate Content-vs-Runtime separation:
- Content layer: human-readable bool fields in RON
- Runtime layer: packed u8 bitmask in TileData (low 8 bits of the u32 packed word)
- Conversion point: tools/map_importer/ (build-time), not the game binary

The conversion is documented in [[terrain-def-l0-vocabulary]] section 6 and in ADR-065 section 4 (Tile → Config mapping strategy).

**Why:** Content authors should not need to understand bitmask arithmetic. The Importer handles the conversion once at build time, and the runtime GridMap reads pre-computed TileFlags directly for performance.

**How to apply:** This pattern (human-friendly content format → machine-friendly runtime format via Importer) should be used whenever the runtime representation requires bit packing or performance-oriented layout that would be confusing in RON.
