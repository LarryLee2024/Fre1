---
name: code-alignment-0621
description: 2026-06-21 alignment of 4 UI architecture docs with actual code implementation at commit 903d039
metadata:
  type: project
---

All 4 UI docs in `docs/06-ui/` were updated to match actual code (commit 903d039) on 2026-06-21.

Key deltas found between design docs and MVP code:

## UiIntent/UiAction/UiCommand/UiEvent
- All type params use `u32`/`i32` primitives, not domain types (SkillId, CharacterId, etc.)
- UiIntent: no SelectGridPos, SelectItem, SelectSaveSlot, Screenshot
- UiAction: no SelectGridPos, ChangeFilter, ChangeSort, ShowContextMenu
- UiCommand: no ChangeSettings
- UiEvent: drastically simpler (6 variants vs 15+ in design)
- into_game_command(): only EndTurn/SaveGame/LoadGame map to Some(GameCommand); all others return None
- See [[application-layer-code-aligned]]

## Screens (MVP state)
- All 3 implemented screens (Battle, MainMenu, Inventory) use hardcoded sample data -- NO ViewModel integration
- Screens use spawn/despawn with marker components (BattleScreen, MainMenuScreen, InventoryScreen)
- Shop/Settings/SaveLoad screens not implemented
- See [[screens-partially-implemented]]

## ViewModel/Projection
- BattleHudVm: hp/mp/ap are f32 (not u32), phase_key is &'static str (not BattlePhaseVm)
- CharacterPanelVm: character_id is u32 (not Option), no buffs/exp/stats
- SkillPanelVm: skills is HashMap<u32, SkillSlotVm> (not Vec)
- UiStore: only 3 fields (battle_hud, character_panel, skill_panel) -- no inventory/shop/quest_log
- Only BattleProjection exists with on_turn_started (real) and on_effect_applied (placeholder)
- Dirty<T> has get_mut() that auto-marks-dirty -- an API not in the original doc
- UiBinding has CharacterLevel, Text, Icon variants not in original doc
- See [[projection-viewmodel-code-aligned]]

## Tests
- 46 pure unit tests across 4 suites (Dirty, ScreenStack, FocusManager, BattleProjection)
- All are stateless/deterministic -- no ECS dependencies
- No Widget/Screen integration or snapshot tests yet
- See [[testing-partially-implemented]]

## Why the gap?
The design docs described an ideal L3-layer architecture with rich ViewModels, full Projection pipeline, and complete enum mappings. The actual code is an MVP that uses simpler types (u32/f32), hardcoded data, and minimal projections. This is intentional -- the architecture is being built incrementally.
