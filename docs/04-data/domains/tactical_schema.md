---
id: domains.tactical.schema.v1
title: Tactical Schema — 战术空间数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance
replay-safe: true
---

# Tactical Schema — 战术空间数据架构

> **领域归属**: Domains — 战术空间层 | **依赖 Schema**: Tag, Terrain | **定义依据**: `docs/02-domain/domains/tactical_domain.md`

---

## 1. Schema Design

### GridPosition（Instance 层）

```rust
struct GridPosition {
    x: i32,
    y: i32,
    layer: i8,          // 层高（0=地面, 1=高地, -1=地下）
}
```

### MovementPoints（Instance 层）

```rust
struct MovementPoints {
    current: f32,
    max: f32,
    consumed: f32,
    movement_type: MovementType,
}

enum MovementType { Walk, Fly, Swim, Climb, Teleport }
```

### Facing（Instance 层）

```rust
struct Facing {
    direction: HexDirection,
}

enum HexDirection { N, NE, SE, S, SW, NW }
```

### FlankingState / CoverState / HighgroundState（Runtime 层 — 瞬时判定结果）

```rust
struct FlankingState { is_flanked: bool, flankers: Vec<EntityId> }
struct CoverState { cover_level: CoverLevel, cover_source: Option<EntityId> }
enum CoverLevel { None, Half, ThreeQuarters, Full }
struct HighgroundState { height_diff: i8, has_advantage: bool }
```

### PathData（Runtime 层）

```rust
struct PathData { waypoints: Vec<GridPosition>, total_cost: f32, is_valid: bool }
```

---

## 2. Layer Summary

Data is primarily Instance/Runtime — positions and movement are tracked per-entity. Grid data itself (map layout, tile data) is managed by Terrain.

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → TerrainSchema | Tile 通行性、高度数据 |
| → TagSchema | MovementType 可表达为 Tag |
| ← CombatSchema | 战斗中的夹击/高地判定 |
| ← TargetingSchema | 射程/视野计算 |

---

## 4. Replay & Save

Movement commands recorded in ReplayLog. Positions serialized in entity save data.
