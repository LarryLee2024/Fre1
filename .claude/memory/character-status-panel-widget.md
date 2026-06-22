---
name: character-status-panel-widget
description: CharacterStatusPanel Organism widget implementation at src/ui/widgets/character_status_panel/ - factory, components, plugin
metadata:
  type: reference
created: 2026-06-22
updated: 2026-06-22
---

# CharacterStatusPanel Widget

Location: `/Users/lf380/Code/Bevy/Fre/src/ui/widgets/character_status_panel/`

Files:
- `components.rs` -- `CharacterStatusPanel` marker component, `CharacterStatusPanelState` (name, hp/mp/ap current/max, status_text, is_active), `CharacterStatusPanelNameLabel`, `CharacterStatusPanelStatusLabel` marker components
- `factory.rs` -- `spawn_character_status_panel()` factory function with `#[allow(clippy::too_many_arguments)]`. Takes flat params (commands, asset_server, theme, name, hp/mp/ap current/max, status_text, is_active). Uses `spawn_panel`, `spawn_text`, `spawn_progress_bar` from primitives, and `spawn_character_portrait` from widgets/character_portrait. Follows same pattern as `character_card/factory.rs`.
- `mod.rs` -- `CharacterStatusPanelPlugin` registering all component types via `register_type`

Registered in `widgets/mod.rs` as `CharacterStatusPanelPlugin` in the plugins tuple.

UI tree:
```
Panel (Card) -- CharacterStatusPanel
  +-- Panel (Basic, Row) -- Top section
  |   +-- CharacterPortrait (placeholder + Active/Inactive border)
  |   +-- Panel (Basic, Column) -- Info column
  |       +-- Text (name, Body, primary)
  |       +-- ProgressBar (HP, Hp, show_label)
  +-- ProgressBar (MP, Mp, show_label)
  +-- ProgressBar (AP, Generic, show_label) -- only if ap_max > 0
  +-- Text (status_text, Caption, centered) -- only if Some
```

MVP: player character full-size mode only. Enemy compact mode and BuffIcon row deferred to future iterations.
Uses `Dirty<CharacterPanelVm>` on container for future refresh system integration.
