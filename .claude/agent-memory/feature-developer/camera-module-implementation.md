---
name: camera-module-implementation
description: Completed Camera base module (Phase 2.1) structure and architecture patterns
metadata:
  type: reference
---

Camera module implementation at `src/infra/camera/` completed 2026-06-21.

## Architecture
- L2 Infrastructure (not UI, not Domain)
- ADR-064 compliant
- No dependency on `core::domains::*` — uses `fn(u64) -> Vec2` resolver closure pattern
- Zero dependency on tile grid types — `TileSize` f32 resource instead

## Module Structure
```
src/infra/camera/
├── mod.rs                          # Module root
├── plugin.rs                       # CameraPlugin + spawn_camera/despawn_camera factories
├── components.rs                   # MainCamera, CameraBounds, CameraShake, CameraInputBlock, IdleTimeout
├── query.rs                        # CameraQuery: world_to_screen, screen_to_world, visible_rect
├── resources.rs                    # UnitPositionResolver, TileSize
├── foundation/
│   ├── mod.rs                      # Re-exports
│   ├── pose.rs                     # CameraPose (value object), TargetPose/CurrentPose wrappers, constants
│   ├── target.rs                   # CameraTarget enum (WorldPos, TilePos, UnitId)
│   ├── request.rs                  # CameraRequest enum (7 variants) with Event derive
│   ├── state.rs                    # CameraState enum (Idle, FreeMove, Follow, Focus) with Component derive
│   └── command.rs                  # CameraCommand enum (serialization target)
├── systems/
│   ├── mod.rs
│   ├── input_handler.rs            # PreUpdate: reads InputState, updates TargetPose
│   ├── state_machine.rs            # Observer + idle_timeout + update_focus
│   ├── movement.rs                 # interpolate_pose + write_to_transform
│   ├── shake.rs                    # apply_shake (deterministic, no RNG)
│   └── bounds.rs                   # clamp_position
└── tests/
    ├── unit/
    │   ├── pose_tests.rs           # 5 lerp tests
    │   └── target_tests.rs         # 5 resolve tests
    └── integration/
        └── state_transition_tests.rs # 5 state machine tests
```

## Schedule
- PreUpdate: input_handler (read InputState)
- Update: process_camera_requests (Observer), idle_timeout, update_focus
- PostUpdate: interpolate_pose -> clamp_position -> apply_shake -> write_to_transform

## State Machine
- Idle <-> FreeMove (user input / idle_timeout)
- Idle/FreeMove -> Follow (CameraRequest::Follow)
- Follow -> Idle (CameraRequest::Unfollow)
- Any -> Focus (request, handled externally via request pipeline)
- Focus ignores all inputs and non-LockInput requests

## Key Patterns
- Observer pattern (On<T>, add_observer, trigger) for all external camera control
- Two immutable queries in observer to avoid borrow conflicts with commands
- CameraTarget uses Copy primitives (Vec2, i32, u64) — no Entity, no Domain types
- Deterministic shake: sin(elapsed * SHAKE_FREQUENCY) — no RNG
- CameraBounds from Vec2 only — no grid/tile knowledge
- Dual-query for target resolution: closure-based resolver pattern
