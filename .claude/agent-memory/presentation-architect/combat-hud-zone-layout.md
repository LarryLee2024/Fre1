---
name: combat-hud-zone-layout
description: Zone-based layout architecture for Combat HUD using 9 screen zones anchored to edges
metadata:
  type: reference
---

The BattleScreen layout was redesigned from a single Column stack to a **9-zone absolute-positioned system** documented in `docs/09-planning/ui-layout-system-plan.md`.

Key rules:
- Zones anchored to screen edges (Top-Left, Top-Center, Top-Right, Bottom-Left, Bottom-Center, Bottom-Right) plus a full-width bottom bar and center game world.
- Zones use `PositionType::Absolute` with offsets derived from `theme.spacing` tokens.
- All widgets live inside zones; no widget is a direct child of `BattleScreenRoot`.
- Visibility is controlled via `Visibility` component (persistent widget), never spawn/despawn on phase transitions.
- Sizing must be multiples of 4px (BASE_UNIT), aligned with existing `UiSpacing` scale.
- A new `UiSizing` resource is needed for zone dimension constants (separate from `Theme` since sizing is theme-agnostic).

Related: [[visibility-rules-pattern]], [[action-chain-patterns]]
