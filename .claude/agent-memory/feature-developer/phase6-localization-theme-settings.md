---
name: phase6-localization-theme-settings
description: Phase 6 completion - localization completion, theme switching, settings persistence
metadata:
  type: project
---

Phase 6 remaining tasks were implemented on 2026-06-21, covering:
- **Task 6.5**: Completed localization by adding missing keys to ja-JP and zz-ZZ FTL files (new-game, load-game, continue, use, equip, drop), added `-ui-inventory` to all FTL files, migrated `inventory_grid/factory.rs` "Inventory" header from `spawn_text` to `spawn_localized_text`, and verified keys.rs auto-generation from build.rs.

- **Task 6.6**: Created `src/ui/theme/switch.rs` with `ThemeVariant` enum (Dark/Light), `as_str()`, `Display`, and `switch_theme()` function. Added `ChangeTheme(ThemeVariant)` to `UiCommand` in `src/ui/application/command.rs`. Registered observer `on_theme_change_command` in `ThemePlugin` that switches theme, updates settings, and calls `save_settings()`.

- **Task 6.7**: Created `src/ui/settings.rs` with `UiSettings` resource (theme, language, show_damage_numbers, battle_speed, tooltip_delay) implementing `Serialize/Deserialize`. Added `load_settings()` and `save_settings()` using RON. Registered in `UiPlugin` via `insert_resource` + `register_type`.

**Key files**:
- `src/ui/theme/switch.rs` - ThemeVariant + switch_theme()
- `src/ui/settings.rs` - UiSettings + load/save
- `src/ui/application/command.rs` - ChangeTheme variant
- `src/ui/theme/mod.rs` - Observer + ThemePlugin updates
- `src/ui/plugin.rs` - UiSettings registration
- `assets/localization/*/ui.ftl` - FTL file updates

**Dependencies**: serde and ron were already in Cargo.toml.

**Pre-existing issues**: Build has pre-existing compilation errors in `replay/events.rs` (duplicate type definitions) and `widgets/mod.rs` (missing `inventory_grid` module declaration) - not related to these changes.
