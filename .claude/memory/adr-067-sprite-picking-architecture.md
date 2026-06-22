---
name: adr-067-sprite-picking-architecture
description: Sprite Picking 架构 ADR: 方案A (SpritePickingPlugin) vs 方案B (Custom Grid Backend) 分析, 推荐方案A分阶段实施
metadata:
  type: reference
---

# ADR-067 Sprite Picking Architecture

**File**: `docs/01-architecture/40-cross-cutting/🔄ADR-067-sprite-picking-architecture.md`

## Core Decision

**Recommendation: Scheme A (SpritePickingPlugin)** over Scheme B (Custom Grid Backend).

Key reasons:
- MVP target: SpritePickingPlugin works in ~30 lines vs ~150 lines for custom backend
- Camera2d + OrthographicProjection explicitly supported by Bevy's sprite_picking backend
- Camera frame timing: `write_to_transform` runs after `TransformPropagate`, causing one-frame GlobalTransform lag. Acceptable for SRPG (discrete clicks, slow camera movement). No camera system changes needed.
- UI penetration: `Pickable::IGNORE` on root UI node resolves the full-screen UI blocking problem
- Phase isolation: `Selection` Resource decouples picking mechanism from business logic
- Phased plan: MVP -> UI ViewModel -> Hover highlight -> optional Grid Backend future

## Module Added

`src/infra/picking/` — infrastructure module for picking configuration (plugin assembly, settings). No picking backend code in MVP phase.

## Architecture Constraints

- infra/picking/ MUST NOT import core::domains::* types
- Picking events MUST use Observer pattern (On<Pointer<Click>>), NOT EventWriter/EventReader
- Selection Resource MUST use BattleUnitId (stable ID), NOT Entity (unstable)
- write_to_transform timing relative to TransformPropagate is Phase 3+ optimization

## Key References

- Bevy 0.19 `bevy_sprite::picking_backend::SpritePickingPlugin` — reads GlobalTransform in PreUpdate
- Bevy 0.19 `bevy_picking::backend` — custom backend interface with PointerHits + RayMap
- `docs/01-architecture/40-cross-cutting/🔄ADR-064-camera-architecture.md` — write_to_transform in PostUpdate
