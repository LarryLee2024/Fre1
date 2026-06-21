---
name: bevy-0.19-api-patterns
description: Bevy 0.19 API patterns discovered during Camera module implementation
metadata:
  type: reference
---

Bevy 0.19 API changes discovered during Camera module (Phase 2.1) implementation at `src/infra/camera/`:

## Observer/Trigger Pattern
- Use `On<T>` not `Trigger<T>` in system parameter: `fn my_fn(trigger: On<CameraRequest>, ...)`
- Register with `app.add_observer(fn)` not `app.observe(fn)` 
- Trigger with `commands.trigger(CameraRequest::...)` — same API
- Observer cannot use `mut` query + `commands` on same entity. Use two immutable queries + commands for all modifications.

## Query API Renames
- `get_single()` → `single()` (returns Result)
- `get_single_mut()` → `single_mut()` (returns Result)
- `get()` → `get()` still works
- `get_mut()` → `get_mut()` still works

## GlobalTransform API
- `compute_matrix()` → `to_matrix()`
- `.inverse()` still works

## Time API
- `delta_seconds()` → `delta().as_secs_f32()`
- `delta_seconds_f64()` → `delta().as_secs_f64()`

## OrthographicProjection
- `OrthographicProjection::default()` removed — use `default_2d()` or just add `Camera2d` bundle which sets it up
- `OrthographicProjection` is NOT queryable as a `Component` in Bevy 0.19 (was removed from component registration)

## Reflect + Registration
- All types passed to `app.register_type::<T>()` need `#[derive(Reflect)]`
- If a struct/enum derives Reflect and contains another type, that type must also derive Reflect
- `Vec2`, `i32`, `u64`, `f32` are primitive Reflect types (no custom derive needed)

## Bundle Limits
- Bevy 0.19 has a bundle component limit of ~10-12 items
- `Camera2d` bundle includes `Camera` + `OrthographicProjection` internally
- `GlobalTransform`, `Visibility`, `InheritedVisibility`, `ViewVisibility` are automatically added

## Entity Access
- `World::entity()` takes `&self` (not `&mut self`) — use `app.world().entity(id)` for read-only access
- `EntityRef::get::<T>()` takes `&self`
- `World::entity_mut()` takes `&mut self` — use `app.world_mut().entity_mut(id)` for mutations
