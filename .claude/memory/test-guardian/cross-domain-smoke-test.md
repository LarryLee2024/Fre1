---
name: cross-domain-smoke-test
description: Wrote tests/scene_smoke.rs cross-domain integration test; added pub re-exports to tactical/mod.rs for external test access
metadata:
  type: project
---

Created the project's first root `tests/` integration test (`tests/scene_smoke.rs`) -- a cross-domain smoke test that verifies `PartySetup -> Combat` state transition produces correct ECS World state (GridMap, TurnQueue, 4 units, 36 grid background entities).

To make this work, added `pub use components::*` and `pub use resources::*` re-exports in `src/core/domains/tactical/mod.rs`, following the same pattern the combat domain already uses.

**Why:** The tactical domain's `components` and `resources` modules were `pub(crate)`, making them inaccessible from `tests/` integration tests. Cross-domain integration tests need these types (GridPos, GridMap, GridLayout) to verify combat spawn behavior.

**How to apply:** When writing future `tests/` integration tests that need to inspect ECS World state, import tactical types from `fre::core::domains::tactical::{GridPos, GridMap, GridLayout}` and combat types from `fre::core::domains::combat::{HitPoints, TurnQueue, UnitIdComponent, CombatParticipant}`. The app test app setup pattern is: `MinimalPlugins + StatesPlugin + ScenePlugin + TestBattlePlugin + init_resource::<Assets<Image>>()`, with the state transition via `StateTransitionQueue`.
