---
name: ui-screen-localization-keys
description: BattleScreen and MainMenuScreen LocalizationKeys designed — 19 keys total (9 existing + 10 new), reorganized under user-requested hierarchical naming
metadata:
  type: project
---

BattleScreen and MainMenuScreen LocalizationKey definitions are documented at `docs/03-content/localization/ui-screen-keys.md`.

**Current state after rewrite (2026-06-22)**: The document was restructured to use the user's requested YAML format and hierarchical key naming (`ui.battle.turn_bar.turn_label`, `ui.battle.action_menu.attack`, `ui.main_menu.title`, etc.). 

**19 design keys total**:
- 9 map to existing FTL entries (code adoption only)
- 10 are new (require FTL entries + code adoption)

**Key decisions in the redesign**:
- `ui.battle.char_panel.character_name` is a field label only; actual character display names should come from CharacterDef.name_key (L2 Entity)
- `ui.battle.char_panel.hp_label` and `mp_label` are DESIGN aliases for the existing general-purpose `ui.hp` / `ui.mp` keys (shared across screens)
- `ui.main_menu.title` / `subtitle` / `version` are all new keys (the existing `ui.main.menu` = "Main Menu" text is different from the game title "Fre")
- The version key is annotated as replaceable by build-time derivation from Cargo.toml/git

**How to apply**: When implementing for P0 delivery, code-adopt the 9 existing keys first (no FTL changes), then add the 10 new FTL entries. See section 4.3-4.4 in the doc for the full implementation order and validation checklist.
