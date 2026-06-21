---
name: action-chain-patterns
description: Documented full click-to-domain action chains for every interactive combat widget
metadata:
  type: reference
---

Every interactive widget follows: ButtonClicked -> ActionComponent match -> UiCommand -> into_game_command() -> CommandQueue -> domain handler.

Existing action chain in `src/ui/application/bridge.rs`: `process_ui_commands` observer handles `On<UiCommand>`, calls `cmd.into_game_command()`, pushes `GameCommand` to `CommandQueue`.

Key mappings documented:
- EndTurnButton -> BattleAction::EndTurn -> UiCommand::EndTurn -> GameCommand::EndTurn
- AttackButton -> ActionType::Attack -> UiCommand::SelectTarget(0) -> None (UI internal targeting mode)
- SkillSlot -> SkillSlotAction::Use -> UiCommand::CastSkill -> GameCommand::CastSpell
- ItemButton -> ActionType::Item -> UiCommand::OpenScreen(Inventory) -> navigation
- WaitButton -> ActionType::Wait -> UiCommand::EndTurn -> GameCommand::EndTurn

Note: `UiCommand::SelectTarget` returns `None` from `into_game_command()` -- targeting is handled entirely within the UI layer (entering a targeting sub-state, then emitting the final command with a target).

Reference: `docs/09-planning/ui-layout-system-plan.md` Section 4.
