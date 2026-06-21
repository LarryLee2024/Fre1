---
name: visibility-rules-pattern
description: Visibility controlled via Visibility component on persistent widgets, driven by BattlePhase state
metadata:
  type: reference
---

Visibility rules for combat widgets are driven by the domain `BattlePhase` (PlayerPhase, EnemyPhase, TransitionPhase, Victory/Defeat) and unit selection state.

Implementation pattern:
- Widgets are **persistent** for the lifetime of BattleScreen.
- A single system reads `State<BattlePhase>` and `BattleHudVm.selected_unit`, writes `Visibility` on zone containers.
- No spawn/despawn on phase transition -- avoids ECS churn.
- ActionMenu hides during enemy phase, CharacterCard hides when no unit selected, etc.
- Sub-states (targeting mode, skill submenu) dim rather than hide widgets to keep layout stable.

Reference: `docs/09-planning/ui-layout-system-plan.md` Section 5.
