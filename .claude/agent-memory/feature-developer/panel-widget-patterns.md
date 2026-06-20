---
name: panel-widget-patterns
description: Panel widget structure, factory pattern, and variant styling approach
metadata:
  type: reference
---

The Panel widget at `src/ui/widgets/panel/` follows the same Factory pattern as Button and ProgressBar. Key implementation details:

- **components.rs**: `PanelVariant` enum (Basic, Card, Modal, Tooltip, List, Group) + `PanelState` Component with `variant`, `padded`, `title` fields. PanelVariant is a `Clone + Copy` enum since it has no data payloads.
- **factory.rs**: `spawn_panel(commands, theme, variant) -> Entity` returns the container entity. Callers add children via `commands.entity(entity).with_children(...)` separately (no closure parameter). Style per variant is computed in a match block that returns `(Node, border_color)` tuple.
- **mod.rs**: `PanelPlugin` only registers `PanelState` via `register_type` -- no systems, no events. Panel is a passive container.
- **Wiring**: Added to `src/ui/widgets/mod.rs` alongside ButtonPlugin, ProgressBarPlugin, TextPlugin in the WidgetsPlugin plugins tuple.

Styling per variant:
- **Basic**: `surface_primary` bg, 1px `border_default` border, `border_radius_sm`, `md` padding
- **Card**: `surface_primary` bg, `border_radius_lg`, `lg` padding, no visible border but `border_default` color set for subtle outline
- **Modal**: `srgba(0,0,0,0.6)` overlay, `PositionType::Absolute` filling parent, centered via `AlignItems::Center` + `JustifyContent::Center`, `lg` padding
- **Tooltip**: `surface_secondary` bg, `border_radius_sm`, `sm` padding
- **List**: `Color::NONE` bg, `Overflow::clip()`, no padding
- **Group**: `surface_primary` bg, `border_radius_sm`, `md` padding

Related: [[widget-button-patterns]], [[progress-bar-patterns]] for sibling widget patterns.
