---
name: ui-screens-implementation-pattern
description: How screens are structured using primitives factories with set_parent_in_place hierarchy
metadata:
  type: reference
---

Screens are the highest UI layer at `src/ui/screens/`. Each screen is a full-page view composed from primitives factories.

Directory structure:
```
screens/mod.rs        -- ScreenPlugin, registers all screens
screens/main_menu/
  mod.rs              -- MenuAction enum, MainMenuScreen marker, spawn_main_menu system
  systems.rs          -- Observer functions (On<ButtonClicked>)
```

Key implementation patterns:
- **Root container**: `spawn_panel(&mut commands, &theme, PanelVariant::Basic)` then override with `Node{ width: Val::Percent(100.0), ... }` for full screen
- **Factories at root level**: All factories (`spawn_text`, `spawn_button`, `spawn_list`) take `&mut Commands`, not `ChildSpawnerCommands`. Call them at root `commands` level, not inside `with_children`.
- **Hierarchy**: Use `commands.entity(child).set_parent_in_place(parent)` to build the tree. This is Bevy 0.19's API (not `set_parent`).
- **Button identification**: Use an enum `Component` (e.g. `MenuAction::NewGame`) rather than matching on `Name` strings.
- **Observer handling**: Register with `app.add_observer(fn_name)`. Function takes `on: On<ButtonClicked>` as first param.
