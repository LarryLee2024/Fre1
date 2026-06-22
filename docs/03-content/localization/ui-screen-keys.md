---
id: 03-content.localization.ui-screen-keys
title: UI Screen LocalizationKeys — BattleScreen / MainMenuScreen
status: draft
owner: content-architect
created: 2026-06-22
updated: 2026-06-22
tags:
  - localization
  - ui
  - battle-screen
  - main-menu
  - localization-key
---

# UI Screen LocalizationKeys — BattleScreen / MainMenuScreen

> Design document defining all LocalizationKeys required for BattleScreen and MainMenuScreen.
> Convention: `ui.{scope}.{id}`, matching the existing `assets/localization/en-US/ui.ftl` namespace.
>
> **Upstream**: Content Compatibility Report (`docs/11-refactor/content-compatibility-report.md`) identified hardcoded text in both screens that must use LocalizationKeys.
>
> **Key**: Some keys already exist in `assets/localization/en-US/ui.ftl` and `generated/keys.rs` (noted as "exists"). Others are new additions requiring FTL entries.

---

## 1. BattleScreen

### 1.1 Turn Bar

```yaml
# Turn counter — displayed as "Turn: 3"
# Existing FTL: NEW (no existing entry)
# Note: existing doc used key_string "ui.battle.turn.text"; this document
#       adopts the user's requested hierarchical naming ui.battle.turn_bar.*
ui.battle.turn_bar.turn_label:
  en: "Turn: { $turn }"
  zh: "回合 { $turn }"
  params:
    - name: turn
      type: u32
      description: "Current turn number (1-based)"

# Player phase label
# Existing FTL: NEW
ui.battle.turn_bar.phase_player:
  en: "Player Turn"
  zh: "玩家回合"

# Enemy phase label
# Existing FTL: NEW
ui.battle.turn_bar.phase_enemy:
  en: "Enemy Turn"
  zh: "敌方回合"
```

### 1.2 Action Menu

The following six keys map to entries that **already exist** in `assets/localization/en-US/ui.ftl` and `src/infra/localization/generated/keys.rs`. Their key strings in the FTL use flat naming (`ui.battle.attack` vs the hierarchical `ui.battle.action_menu.attack` shown below). Code adoption only -- replace hardcoded strings with `loc::ui::BATTLE_ATTACK`, `loc::ui::BATTLE_DEFEND`, etc.

```yaml
# Attack action label
# FTL: -ui-battle-attack | Key string: ui.battle.attack
# Rust constant: loc::ui::BATTLE_ATTACK
# Status: EXISTS (code adoption only)
ui.battle.action_menu.attack:
  en: "Attack"
  zh: "攻击"

# Defend action label
# FTL: -ui-battle-defend | Key string: ui.battle.defend
# Rust constant: loc::ui::BATTLE_DEFEND
# Status: EXISTS (code adoption only)
ui.battle.action_menu.defend:
  en: "Defend"
  zh: "防御"

# Skill action label
# FTL: -ui-battle-skill | Key string: ui.battle.skill
# Rust constant: loc::ui::BATTLE_SKILL
# Status: EXISTS (code adoption only)
ui.battle.action_menu.skill:
  en: "Skill"
  zh: "技能"

# Item action label
# FTL: -ui-battle-item | Key string: ui.battle.item
# Rust constant: loc::ui::BATTLE_ITEM
# Status: EXISTS (code adoption only)
ui.battle.action_menu.item:
  en: "Item"
  zh: "道具"

# Wait action label
# FTL: -ui-battle-wait | Key string: ui.battle.wait
# Rust constant: loc::ui::BATTLE_WAIT
# Status: EXISTS (code adoption only)
ui.battle.action_menu.wait:
  en: "Wait"
  zh: "等待"
```

### 1.3 Top Bar

```yaml
# End Turn button label
# FTL: -ui-battle-end-turn | Key string: ui.battle.end.turn
# Rust constant: loc::ui::BATTLE_END_TURN
# Status: EXISTS (code adoption only)
ui.battle.top_bar.end_turn:
  en: "End Turn"
  zh: "结束回合"
```

### 1.4 Character Panel

The HP and MP labels **already exist** as general-purpose keys (`ui.hp`, `ui.mp`) shared across multiple screens. The level prefix is a new battle-specific key.

```yaml
# HP label on character card
# FTL: -ui-hp | Key string: ui.hp
# Rust constant: loc::ui::HP
# Status: EXISTS (code adoption only)
# Note: general-purpose key, not battle-specific
ui.battle.char_panel.hp_label:
  en: "HP"
  zh: "生命"

# MP label on character card
# FTL: -ui-mp | Key string: ui.mp
# Rust constant: loc::ui::MP
# Status: EXISTS (code adoption only)
# Note: general-purpose key, not battle-specific
ui.battle.char_panel.mp_label:
  en: "MP"
  zh: "魔力"

# Level prefix on character card — "Lv.{n}"
# FTL: NEW (distinct from general-purpose ui.level = "Level")
ui.battle.char_panel.level_prefix:
  en: "Lv."
  zh: "Lv."

# Character name label on character card — "Name:"
# FTL: NEW
# Note: This labels the character name field on the card.
# The actual character display name should be sourced from
# CharacterDef.name_key (L2 Entity), not a UI screen key.
# This key serves as the field label, not the character's name value.
ui.battle.char_panel.character_name:
  en: "Name"
  zh: "名称"
```

---

## 2. MainMenuScreen

### 2.1 Title Area

```yaml
# Game title — "Fre"
# FTL: NEW
ui.main_menu.title:
  en: "Fre"
  zh: "Fre"

# Subtitle — "A Bevy SRPG"
# FTL: NEW
# Note: VOLATILE KEY. Tagline may change during development.
# The key name is stable regardless of value changes.
ui.main_menu.subtitle:
  en: "A Bevy SRPG"
  zh: "一款 Bevy SRPG 游戏"

# Version string — "v0.1.0"
# FTL: NEW
# Note: Alternative: derive from Cargo.toml / git describe at build time.
# This key is a fallback for the static case; if build-time derivation
# is implemented, this key may be omitted.
ui.main_menu.version:
  en: "v0.1.0"
  zh: "v0.1.0"
```

### 2.2 Buttons

All four button keys **already exist** as general-purpose UI keys used across screens.

```yaml
# New Game button
# FTL: -ui-new-game | Key string: ui.new.game
# Rust constant: loc::ui::NEW_GAME
# Status: EXISTS (code adoption only)
ui.main_menu.new_game:
  en: "New Game"
  zh: "新游戏"

# Load Game button
# FTL: -ui-load-game | Key string: ui.load.game
# Rust constant: loc::ui::LOAD_GAME
# Status: EXISTS (code adoption only)
ui.main_menu.load_game:
  en: "Load Game"
  zh: "读取存档"

# Settings button
# FTL: -ui-settings | Key string: ui.settings
# Rust constant: loc::ui::SETTINGS
# Status: EXISTS (code adoption only)
ui.main_menu.settings:
  en: "Settings"
  zh: "设置"
```

---

## 3. Key Register

### 3.1 All Keys

| # | Key string | Screen | Widget Region | FTL Status | Rust Constant |
|---|-----------|--------|--------------|------------|---------------|
| 1 | `ui.battle.turn_bar.turn_label` | Battle | Turn Bar | NEW | `loc::ui::BATTLE_TURN_BAR_TURN_LABEL` |
| 2 | `ui.battle.turn_bar.phase_player` | Battle | Turn Bar | NEW | `loc::ui::BATTLE_TURN_BAR_PHASE_PLAYER` |
| 3 | `ui.battle.turn_bar.phase_enemy` | Battle | Turn Bar | NEW | `loc::ui::BATTLE_TURN_BAR_PHASE_ENEMY` |
| 4 | `ui.battle.action_menu.attack` | Battle | Action Menu | EXISTS | `loc::ui::BATTLE_ATTACK` |
| 5 | `ui.battle.action_menu.defend` | Battle | Action Menu | EXISTS | `loc::ui::BATTLE_DEFEND` |
| 6 | `ui.battle.action_menu.skill` | Battle | Action Menu | EXISTS | `loc::ui::BATTLE_SKILL` |
| 7 | `ui.battle.action_menu.item` | Battle | Action Menu | EXISTS | `loc::ui::BATTLE_ITEM` |
| 8 | `ui.battle.action_menu.wait` | Battle | Action Menu | EXISTS | `loc::ui::BATTLE_WAIT` |
| 9 | `ui.battle.top_bar.end_turn` | Battle | Top Bar | EXISTS | `loc::ui::BATTLE_END_TURN` |
| 10 | `ui.battle.char_panel.hp_label` | Battle | Character Panel | EXISTS | `loc::ui::HP` |
| 11 | `ui.battle.char_panel.mp_label` | Battle | Character Panel | EXISTS | `loc::ui::MP` |
| 12 | `ui.battle.char_panel.level_prefix` | Battle | Character Panel | NEW | `loc::ui::BATTLE_CHAR_PANEL_LEVEL_PREFIX` |
| 13 | `ui.battle.char_panel.character_name` | Battle | Character Panel | NEW | `loc::ui::BATTLE_CHAR_PANEL_CHARACTER_NAME` |
| 14 | `ui.main_menu.title` | MainMenu | Title Area | NEW | `loc::ui::MAIN_MENU_TITLE` |
| 15 | `ui.main_menu.subtitle` | MainMenu | Title Area | NEW | `loc::ui::MAIN_MENU_SUBTITLE` |
| 16 | `ui.main_menu.version` | MainMenu | Title Area | NEW | `loc::ui::MAIN_MENU_VERSION` |
| 17 | `ui.main_menu.new_game` | MainMenu | Buttons | EXISTS | `loc::ui::NEW_GAME` |
| 18 | `ui.main_menu.load_game` | MainMenu | Buttons | EXISTS | `loc::ui::LOAD_GAME` |
| 19 | `ui.main_menu.settings` | MainMenu | Buttons | EXISTS | `loc::ui::SETTINGS` |

### 3.2 Summary

| Category | Count | Action |
|----------|-------|--------|
| Existing keys (code adoption only) | 9 | Replace hardcoded strings with `loc::ui::*` constants |
| New keys (FTL + code adoption) | 10 | Add to all locale `.ftl` files, regenerate `keys.rs`, adopt in code |
| **Total** | **19** | |

Note: The count of 9 "existing" keys counts distinct FTL entries. `ui.battle.action_menu.attack` maps to existing `ui.battle.attack`, `ui.battle.char_panel.hp_label` maps to existing `ui.hp`, etc. The actual FTL entries needed are fewer than the design keys because some design keys share the same underlying FTL entry (aliases for the same generated constant).

### 3.3 FTL Mapping Notes

The following existing FTL entries are shared between this document's hierarchical design keys and other screens:

| Design Key | FTL Entry | Also Used By |
|-----------|-----------|-------------|
| `ui.battle.char_panel.hp_label` | `-ui-hp` → `ui.hp` | Party screen, Shop screen, Inventory |
| `ui.battle.char_panel.mp_label` | `-ui-mp` → `ui.mp` | Party screen, Shop screen, Inventory |
| `ui.main_menu.new_game` | `-ui-new-game` → `ui.new.game` | Continue prompt, Title screen variants |
| `ui.main_menu.load_game` | `-ui-load-game` → `ui.load.game` | Save/Load screen |
| `ui.main_menu.settings` | `-ui-settings` → `ui.settings` | In-game pause menu, Settings screen |

---

## 4. Implementation Guidance

### 4.1 Parameterized Key: `ui.battle.turn_bar.turn_label`

The `{$turn}` parameter must be passed at the call site:

```rust
// Correct usage:
commands.spawn(LocalizedText::with_params(
    loc::ui::BATTLE_TURN_BAR_TURN_LABEL,  // or newly generated constant
    &[("turn", turn_number.to_string())],
));
```

### 4.2 Character Name Key: `ui.battle.char_panel.character_name`

This key labels the character name **field** on the character card. The actual character display value should ultimately come from `CharacterDef.name_key` (L2 Entity layer). This key is a transitional measure until the full CharacterDef localization pipeline is operational.

### 4.3 Implementation Order

1. **P0**: Code-adopt the 9 existing keys (replace hardcoded strings with `loc::ui::*` constants directly -- no FTL changes needed)
2. **P0**: Add 10 new keys to `assets/localization/en-US/ui.ftl`
3. **P0**: Add 10 new keys to `assets/localization/zh-CN/ui.ftl`
4. **P1**: Add new keys to `assets/localization/ja-JP/ui.ftl` and `assets/localization/zz-ZZ/ui.ftl`
5. **Build**: Trigger `build.rs` regeneration of `generated/keys.rs`
6. **Code**: Adopt new constants in screen code

### 4.4 Validation Checklist

- [ ] All 9 existing keys are referenced via `loc::ui::*` constants (no hardcoded strings remain)
- [ ] All 10 new keys have FTL entries in en-US ui.ftl
- [ ] All 10 new keys have corresponding entries in zh-CN ui.ftl
- [ ] `generated/keys.rs` includes all new keys
- [ ] `cargo build` succeeds (no compile errors from missing constants)
- [ ] Startup validation passes with no KeyNotFound errors
- [ ] Fake locale (zz-ZZ) renders all screen text with bracket-wrapped pseudo-translation (catches any remaining hardcoded strings)
