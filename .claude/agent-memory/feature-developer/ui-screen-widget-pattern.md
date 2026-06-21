---
name: ui-screen-widget-pattern
description: Screen spawning/despawning + widget factory + observer registration patterns used in the UI layer
metadata:
  type: reference
---

## Screen Pattern

Each screen in `src/ui/screens/<name>/` follows this structure:

- `mod.rs` — Screen marker component (`XxxScreen`), `spawn_xxx_screen()` (Startup system), `despawn_xxx_screen()` (cleanup query)
- `systems.rs` (optional) — Observer functions for `On<ButtonClicked>` that map action components to `UiCommand`

Screens register in `src/ui/screens/mod.rs` via `ScreenPlugin`:
  - `.register_type::<XxxScreen>()`
  - `.add_systems(Startup, spawn_xxx_screen)`
  - `.add_observer(on_xxx_button_clicked)`

## Widget Pattern

Each widget in `src/ui/widgets/<name>/` follows this structure:

- `components.rs` — Marker components, action enums, state structs
- `factory.rs` — `spawn_xxx_widget()` pure factory function composing primitives
- `mod.rs` — `XxxWidgetPlugin` registering types and systems

Widgets register in `src/ui/widgets/mod.rs` via `WidgetsPlugin`:
  - Module declaration
  - Plugin import and addition to tuple

## Primitives Factories (import from `crate::ui::primitives::*`)

- `spawn_panel(commands, theme, PanelVariant)` → container Entity
- `spawn_text(commands, asset_server, theme, content, TextVariant)` → text Entity  
- `spawn_localized_text(commands, asset_server, theme, key, fallback, TextVariant)` → localized text Entity
- `spawn_button(commands, theme, label, ButtonVariant)` → button Entity
- `spawn_localized_button(commands, theme, key, default_label, ButtonVariant)` → localized button Entity
- `spawn_list(commands, theme, ListVariant)` → list container Entity

## Localization Keys

Generated in `src/infra/localization/generated/keys.rs` as `loc::ui::XXX` constants. UI-related keys include: USE, EQUIP, DROP, GOLD, CLOSE, BACK, CONFIRM, etc.
