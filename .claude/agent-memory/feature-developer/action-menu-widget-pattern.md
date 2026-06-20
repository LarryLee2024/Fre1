---
name: action-menu-widget-pattern
description: ActionMenu composite widget implementation at src/ui/widgets/action_menu/
metadata:
  type: reference
---

ActionMenu implemented at `/Users/lf380/Code/Bevy/Fre/src/ui/widgets/action_menu/`. A vertical list of action buttons (Attack, Defend, Skill, Item, Wait) shown in battle.

Pattern:
- `components.rs` -- `ActionType` (Component enum), `ActionMenuItem` (data struct), `ActionMenuState` (Component with `Vec<ActionMenuItem>`)
- `factory.rs` -- `spawn_action_menu(commands, theme)` using `spawn_list(Vertical)` as container + `spawn_button` for each action with different `ButtonVariant` (Attack=Primary, Defend=Secondary, Skill=Primary, Item=Secondary, Wait=Ghost). Each button gets `ActionType` component inserted. Hierarchy via `commands.entity(child).set_parent_in_place(container)`.
- `systems.rs` -- `action_menu_sync_system` with `Changed<ActionMenuState>` filter, syncs `enabled` field from `ActionMenuItem` to child button `ButtonState.disabled`.
- `mod.rs` -- `ActionMenuPlugin` registers `ActionMenuState` + `ActionType` types and adds the sync system to Update.
- Wired into `widgets/mod.rs` by adding `pub mod action_menu;` + `ActionMenuPlugin` in the plugin tuple.

Key constraints followed: only primitives factories (no direct Node/Button/Interaction), `set_parent_in_place` for hierarchy (Bevy 0.19), all from Theme.
