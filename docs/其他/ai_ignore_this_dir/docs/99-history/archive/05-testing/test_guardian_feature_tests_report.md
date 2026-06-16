# Feature Tests Report (test-guardian Part A + B)

**Date**: 2026-06-13
**Scope**: `tests/feature/campaign.rs` (Part A: Content Loading) + `tests/feature/ui_screens.rs` (Part B: UI Flow)
**Agent**: test-guardian

---

## Summary

| Item | Status |
|------|--------|
| Part A: Campaign content loading tests | ✅ 5/5 passed |
| Part B: UI screen flow tests | ✅ 12/12 passed |
| Full regression suite | ✅ 498 passed, 0 failed |

---

## Part A: Content Loading (`tests/feature/campaign.rs`)

### Test IDs: FT-CAMP-001 ~ FT-CAMP-005

| ID | Test | Assertions | Result |
|----|------|-----------|--------|
| FT-CAMP-001 | `enemy_goblin_leader_deserialization` | RON deserialization, faction, skill_ids, attributes | ✅ |
| FT-CAMP-002 | `tutorial_level_deserialization_and_conversion` | LevelConfigDef fields, terrain map, unit deployment, LevelConfig.from_def | ✅ |
| FT-CAMP-003 | `player_archer_skill_ids_contains_pierce` | RON deserialization, skill_ids contains "pierce" | ✅ |
| FT-CAMP-004 | `player_mage_skill_ids_contains_heal` | RON deserialization, skill_ids contains "heal" | ✅ |
| FT-CAMP-005 | `campaign_001_deserialization` | CampaignDef RON deserialization, stages, victory conditions | ✅ |

### Fixes applied:
- `HashMap<AttributeKind, f32>` keys use `AttributeKind` enum instead of string literals
- Save `height`/`width` before `def` is moved into `LevelConfig::from_def`
- Terrain coordinates corrected to match actual `tutorial.ron` data: `(3,1)` for forest, `(7,1)` for water

---

## Part B: UI Screen Flow (`tests/feature/ui_screens.rs`)

### Test IDs: UI-SCR-001 ~ UI-SCR-010

| ID | Test | Assertions | Result |
|----|------|-----------|--------|
| UI-SCR-001 | `main_menu_entities_spawned` | MainMenuScreen entity count = 1 after OnEnter | ✅ |
| UI-SCR-002 | `main_menu_cleaned_on_exit` | MainMenuScreen entity count = 0 after transition | ✅ |
| UI-SCR-003 | `start_game_triggers_state_transition` | AppState::LevelSelect + CampaignProgress populated | ✅ |
| UI-SCR-004 | `level_select_entities_spawned` | LevelSelectScreen entity, ViewModel stages | ✅ |
| UI-SCR-005 | `level_select_confirm_enters_ingame` | AppState::InGame, LevelSelectScreen cleaned | ✅ |
| UI-SCR-006 | `level_select_back_returns_to_main_menu` | AppState::MainMenu, LevelSelectScreen cleaned | ✅ |
| UI-SCR-007 | `game_over_victory_ui` | GameOverScreen entity, GameResultView::Victory | ✅ |
| UI-SCR-008 | `game_over_defeat_ui` | GameOverScreen entity, GameResultView::Defeat | ✅ |
| UI-SCR-009 | `game_over_retry_returns_to_ingame` | AppState::InGame, GameOverState::Playing, cleanup | ✅ |
| UI-SCR-010 | `e2e_full_flow` | Full cycle: MainMenu→LevelSelect→InGame→GameOver→MainMenu | ✅ |
| — | `game_result_view_default_is_victory` | Default GameResultView is Victory (unit test) | ✅ |
| — | `level_select_state_default_is_empty` | Default LevelSelectState has 0 stages (unit test) | ✅ |

### Fixes applied:
- `entity_count()` function signature: `&App` → `&mut App` (needs mutable `World` for `query()`)
- State comparisons: `State::new(AppState::X)` → `AppState::X` (deref to inner value)
- `CnFont` resource: inserted directly after plugins (MinimalPlugins doesn't run Startup)
- `Font` asset type: registered via `app.init_asset::<Font>()` (MinimalPlugins skips TextPlugin)

---

## Source Code Changes for Testability

| File | Change |
|------|--------|
| `tests/feature.rs` | Added `campaign` and `ui_screens` module declarations |
| `tests/feature/campaign.rs` | New file — 5 content loading tests |
| `tests/feature/ui_screens.rs` | New file — 12 UI screen flow tests |
| `src/character/mod.rs` | `mod template` → `pub mod template` |
| `src/ui/mod.rs` | `mod screens` → `pub mod screens` |
| `src/ui/screens/main_menu.rs` | Button markers `pub(crate)` → `pub` |
| `src/ui/screens/level_select.rs` | Button markers `pub(crate)` → `pub` |
| `src/ui/screens/game_over.rs` | Button markers `pub(crate)` → `pub` |

---

## Test Pyramid Compliance

| Layer | Count | Percentage |
|-------|-------|------------|
| Unit tests (`src/` inline) | 466 | ~93.6% |
| Feature tests (`tests/feature/`) | 31 | ~6.2% |
| Scenario tests (`tests/scenario/`) | 1 | ~0.2% |

Target: Unit 70% / Integration 20% / E2E 10% — more integration and E2E tests needed.

---

## Constraints Compliance

- ✅ **🟥禁止测试实现细节** — Tests validate behavior (entity counts, state transitions, data values)
- ✅ **🟥禁止像素级断言** — No pixel-level assertions
- ✅ **🟥禁止修改领域规则让测试通过** — RON data corrected, not rules
- ✅ **🟥禁止删除已有测试** — All 498 existing tests pass
- ✅ **🟥禁止非确定性数据** — Hardcoded test data, no randomness
- ✅ UI commands sent via `UiCommand` Message pattern (or direct state/resource manipulation)

---

## Agent Handoff

**Next steps for follow-up sessions:**
- Add more E2E tests (campaign flow, inventory integration, skill resolution)
- Consider scenario tests for multi-turn battle flows
- Address pre-existing warnings (unused imports, dead code, noop method calls)
