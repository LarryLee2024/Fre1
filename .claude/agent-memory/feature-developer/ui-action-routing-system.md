---
name: ui-action-routing-system
description: UiAction routing system connecting buttons to UiCommand through Bevy 0.19 trigger/observer pattern in src/ui/application/
metadata:
  type: reference
---

# UiAction Routing System

## Location
- `src/ui/application/` — UiAction, UiCommand, UiEvent enums
- `src/ui/navigation/` — ScreenType enum
- `src/ui/primitives/button/systems.rs` — Dual-channel trigger (ButtonClicked + UiAction::Click)
- `src/ui/screens/main_menu/systems.rs` — ButtonClicked → UiCommand mapping (方案A)
- `src/ui/screens/battle/systems.rs` — ButtonClicked → UiCommand mapping (方案A)

## Architecture

**Original Observer-only pattern:** Button system → `commands.trigger(ButtonClicked)` → `On<ButtonClicked>` observer → direct action (no command layer)

**New dual-channel pattern:** Button system → `commands.trigger(ButtonClicked) + commands.trigger(UiAction::Click)` → Screen observers → `commands.trigger(UiCommand)` → future UiCommand handler systems

## Bevy 0.19 Trigger API Used

- `commands.trigger(T)` to emit events (NOT `EventWriter<T>::send()`)
- `fn handler(on: On<T>, ...)` as observer function signature (NOT `EventReader<T>`)
- `app.add_observer(handler)` to register observers (NOT `app.add_event::<T>()`)
- No `add_event`, `EventWriter`, or `EventReader` available in Bevy 0.19 — everything goes through triggers/observers

## 方案A: ButtonClicked → UiCommand Direct Mapping

Since `UiAction::Click` doesn't carry entity info (and can't — it's a unit variant), button-specific commands (like NewGame, OpenScreen, EndTurn) use the existing ButtonClicked trigger which carries the entity. The observer queries the entity's action component (MenuAction, BattleAction) and maps it to the appropriate UiCommand.

## Enums Created

- **UiAction** (`action.rs`) — Click, Confirm, Cancel, Dismiss, SelectSkill(u32), SelectItem(u32), SelectCharacter(u32), Toggle(bool), ChangeTab(usize), TextChanged(String), TextConfirmed(String), Custom(String)
- **UiCommand** (`command.rs`) — CastSkill(u32,u32), SelectTarget(u32), EndTurn, MoveToPosition(i32,i32), UseItem(u32), EquipItem(u32,u32), DropItem(u32), AcceptQuest(u32), AbandonQuest(u32), BuyItem(u32,u32), SellItem(u32,u32), SaveGame(u32), LoadGame(u32), TogglePause, OpenScreen(ScreenType), CloseScreen, NewGame
- **UiEvent** (`event.rs`) — ViewModelUpdated(&'static str), ScreenPushed(ScreenType), ScreenPopped(ScreenType), ScreenReplaced(ScreenType,ScreenType), NavigationError(String), ThemeChanged(String)
- **ScreenType** (`screen_type.rs`) — MainMenu, Battle, Inventory, Shop, Settings, SaveLoad

All derive `Event + Debug + Clone` (NOT Reflect — not needed for triggers, and `&'static str` in UiEvent makes Reflect complex).

## Key Files
- `/src/ui/application/mod.rs` — Module root, re-exports UiAction, UiCommand, UiEvent
- `/src/ui/navigation/mod.rs` — Module root, re-exports ScreenType
- `/src/ui/mod.rs` — Top-level re-exports for application and navigation modules
- `/src/ui/primitives/button/systems.rs` — Dual-channel trigger emission
- `/src/ui/screens/main_menu/systems.rs` — MainMenu action handlers
- `/src/ui/screens/battle/systems.rs` — Battle action handler
- `/src/ui/screens/mod.rs` — ScreenPlugin with observer registrations
