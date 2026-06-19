---
name: ui-implementation-patterns
description: docs/06-ui/01-architecture/implementation-patterns.md defines 4+1 Bevy ECS patterns bridging What-to-build docs to How-to-implement code
metadata:
  type: project
---

The "bridge" document at `docs/06-ui/01-architecture/implementation-patterns.md` defines four structural patterns and one lifecycle overview that tell feature-developers how to translate upstream Widget/Screen/ViewModel/Overlay contracts into Bevy ECS Plugin code:

1. **Widget Plugin pattern** — each Widget Contract = 1 Plugin registering Props Component, State Component, UiAction Event, spawn_on_command, on_dirty, on_interaction, cleanup_on_despawn. Example: ProgressBar with full skeleton in pseudocode.
2. **Screen Plugin pattern** — each Screen = 1 Plugin with OnEnter(State) spawn_screen, OnExit(State) despawn_screen, UiAction→UiCommand conversion, and overlay dependency registration. Example: BattleScreen with full entity tree structure diagram.
3. **ViewModel Update Cycle** — complete 8-step lifecycle from Domain Event → Observer → Projection → Dirty<T> → on_dirty System → UI Node update → sleep. Example: BattleHudVm + HpBar with every step shown.
4. **Overlay Trigger pattern** — Cue/Hover/System → Overlay Service → ViewModel → Spawn → Timer → Despawn. Example: DamageTextOverlay consuming CueType::Popup.
5. **ASCII lifecycle overview** — 7 vertical layers (Input/Screen/Command/Projection/Update/Overlay/Exit) with full arrow diagram.

Ends with a **Feature Developer Handoff Checklist** — ordered steps (Contract → ViewModel → Projection → Widget Plugin → Composite → Screen Plugin → Overlay → UiCommand) and a troubleshooting decision tree.

**Why:** The "What" docs (widget-atoms, widget-composites, screens, focus-binding, application-layer, ADR-055) all existed but feature-developers had no "How" guide connecting them to ECS implementation.

**How to apply:** When a feature-developer asks how to implement a new Widget, Screen, or Overlay, point them to this document's pattern + the relevant upstream doc's What. Do not let them jump straight to coding without first checking the pattern structure.
