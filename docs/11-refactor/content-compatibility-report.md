---
id: 11-refactor.content-compatibility-report
title: Content Architecture Compatibility Report for Screen Spec
status: draft
owner: content-architect
created: 2026-06-22
tags:
  - content
  - screen-spec
  - widget-id
  - localization-key
  - data-flow
  - registry
---

# Content Architecture Compatibility Report for Screen Spec

## Executive Summary

The UI Screen Specification refactoring (07-specs/) is **substantially compatible** with the existing Content Platform architecture. Three dimensions require specific attention before P0 delivery: a Widget-ID-to-Def mapping document, missing LocalizationKeys for BattleScreen text, and a minor annotation to the Projection pattern. No architectural changes to 03-content/ are needed.

---

## 1. Def Registry Mapping Review

### 1.1 Classification: Three Categories of Widget IDs

Widget IDs from the plan (`07-specs/references/widget-id-map.md`) fall into three categories relative to the Def Registry:

**Category A: Widget IDs that display Def-derived data (indirectly via Projection)**

These widget IDs correspond to fields in a ViewModel (`UiStore` field), which in turn is populated by a Projection that queries `DefRegistry<T>`. The widget itself never reads the Def Registry directly -- only the Projection does.

| Widget ID | ViewModel Field | Def Registry | Projection |
|-----------|----------------|-------------|------------|
| `battle_hp_bar` | `BattleHudVm.hp / max_hp` | N/A (runtime instance data) | BattleProjection |
| `battle_mp_bar` | `BattleHudVm.mp / max_mp` | N/A (runtime instance data) | BattleProjection |
| `battle_buff_icons_0` | `BattleHudVm.buffs[0]` | `DefRegistry<BuffDef>` (via `BuffVm`) | BattleProjection |
| `battle_skill_slot_0` | `SkillPanelVm.skills[0]` | `DefRegistry<SpellDef>` (via `SkillSlotVm`) | BattleProjection |
| `battle_character_name` | `CharacterPanelVm.name_key` | `DefRegistry<CharacterDef>` | CharacterPanelProjection |
| `inventory_item_grid` | `InventoryVm.items` | `DefRegistry<ItemDef>` | InventoryProjection |
| `quest_entry_title` | `QuestLogVm.entries[0]` | `DefRegistry<QuestDef>` | QuestProjection |

**Compatibility verdict**: Fully compatible. The existing pipeline (`projection-viewmodel.md` 8.1) already defines `DefRegistry → Projection → ViewModel` as the standard flow. Widget IDs that display Def data map to ViewModel fields, not directly to Def IDs.

**Recommendation**: The `widget-id-map.md` should add a `def_type` column for Category A widget IDs, documenting which Def Registry they ultimately consume data from. This aids impact analysis ("which widgets break when I change SpellDef?").

Example addition to the proposed YAML in the plan:

```yaml
# Widget ID → Def Type reference (Category A only)
def_dependencies:
  battle_hp_bar:             ~                 # Runtime HP, no Def
  battle_buff_icons_0:       BuffDef           # Via BuffVm → Projection → DefRegistry<BuffDef>
  battle_skill_slot_0:       SpellDef          # Via SkillSlotVm → Projection → DefRegistry<SpellDef>
  battle_character_name:     CharacterDef      # Via CharacterPanelVm → Projection → DefRegistry<CharacterDef>
  inventory_item_grid:       ItemDef           # Via InventoryVm → Projection → DefRegistry<ItemDef>
```

---

**Category B: Widget IDs that emit Def-referencing Commands**

These widget IDs emit `UiCommand` events that carry typed IDs (`SkillId`, `ItemId`, etc.). The widget itself does not resolve the Def -- it only triggers the command. The Def resolution happens downstream in the Domain.

| Widget ID | Emitted Command | ID Type | Def Registry |
|-----------|----------------|---------|-------------|
| `battle_attack_btn` | `UiCommand::CastSkill(skill_id, target_id)` | `SkillId` | `DefRegistry<SpellDef>` (resolved by Domain) |
| `battle_skill_btn` | `UiCommand::CastSkill(skill_id, target_id)` | `SkillId` | `DefRegistry<SpellDef>` |
| `battle_item_btn` | `UiCommand::UseItem(item_id, target_id)` | `ItemId` | `DefRegistry<ItemDef>` |
| `inventory_item_slot_0` | `UiCommand::UseItem(item_id)` | `ItemId` | `DefRegistry<ItemDef>` |
| `shop_buy_btn` | `UiCommand::BuyItem(item_id, quantity)` | `ItemId` | `DefRegistry<ItemDef>` |

**Compatibility verdict**: Fully compatible. The command emission pattern exists in `screens.md` 2.4 and 4.4. No Content Platform change needed.

---

**Category C: Pure UI containers with no Def mapping**

These widget IDs have zero dependency on any Def type. They are structural containers or state indicators.

| Widget ID | UiBinding | Def Dependency |
|-----------|-----------|---------------|
| `battle_root` | `UiBinding::None` | None |
| `battle_top_bar` | `UiBinding::None` | None |
| `battle_turn_indicator` | `UiBinding::Turn` | None (reads BattleHudVm.turn_number) |
| `battle_phase_label` | `UiBinding::Phase` | None (reads BattleHudVm.phase_key) |
| `battle_end_turn_btn` | `UiBinding::None` | None |
| `battle_area` | `UiBinding::None` | None |
| `battle_action_menu` | `UiBinding::None` | None |
| `battle_defend_btn` | `UiBinding::None` | None |
| `battle_wait_btn` | `UiBinding::None` | None |
| `main_menu_root` | `UiBinding::None` | None |
| `main_menu_title_text` | `UiBinding::Text` | None |
| `main_menu_version_text` | `UiBinding::None` | None |

**Compatibility verdict**: Fully compatible. No Content Platform involvement.

### 1.2 Existing UiBinding Coverage for New Widget IDs

The existing `UiBinding` enum (`focus-binding.md` 4.2) defines 17 variants. Mapping against the plan's widget IDs:

| UiBinding Variant | Mapped Widget IDs | Status |
|------------------|-------------------|--------|
| `Hp` | `battle_hp_bar`, `battle_hp_text` | Exists |
| `Mp` | `battle_mp_bar`, `battle_mp_text` | Exists |
| `Ap` | `battle_ap_bar` | Exists |
| `Turn` | `battle_turn_indicator` | Exists |
| `Phase` | `battle_phase_label` | Exists |
| `Level` | `battle_character_level` | Exists |
| `SkillSlot(u8)` | `battle_skill_slot_0`, `battle_skill_slot_1`, etc. | Exists |
| `ItemSlot(u8)` | `inventory_item_slot_0`, etc. | Exists |
| `Gold` | `shop_gold_display`, `inventory_gold` | Exists |
| `QuestEntry(u16)` | `quest_entry_title` | Exists |
| `Text` | `main_menu_title_text`, generic text widgets | Exists |

**Gap identified**: Widgets like `end_turn_btn`, `battle_attack_btn`, `battle_defend_btn`, `battle_wait_btn` use `UiBinding::None` in the plan's example. This is correct because these are interactive buttons that do not display bound data. However, if they need to be targeted by query (e.g., enable/disable all action buttons), adding dedicated UiBinding variants (`ActionAttack`, `ActionDefend`, `ActionWait`) would be more performant than scanning for `UiBinding::None` + a secondary component marker. This is a @presentation-architect decision, not a content architecture concern.

---

## 2. LocalizationKey Coverage Review

### 2.1 Existing LocalizationKey Coverage

The existing schema (`localization_schema.md` 3.1 and `theme-localization.md` 4.2) defines the format as:

```
ui.<scope>.<id>.<suffix>
```

Example keys mentioned in existing docs:

| Key | File | Status |
|-----|------|--------|
| `ui.battle.end_turn` | theme-localization.md 4.2 | Defined |
| `ui.battle.victory` | theme-localization.md 4.2 | Defined |
| `ui.battle.phase.player` | projection-viewmodel.md 7 | Defined |
| `ui.battle.attack` | localization_schema.md 3.6 (generated keys example) | Defined |
| `ui.battle.defend` | localization_schema.md 3.6 (generated keys example) | Defined |
| `ui.battle.damage_dealt.text` | localization_schema.md 3.6 (generated keys example) | Defined |
| `ui.battle.heal_received.text` | localization_schema.md 3.6 (generated keys example) | Defined |
| `ui.battle.unit_died.text` | localization_schema.md 3.6 (generated keys example) | Defined |
| `ui.menu.settings` | localization_schema.md 3.6 | Defined |
| `ui.menu.quit` | localization_schema.md 3.6 | Defined |
| `ui.inventory.empty_slot` | theme-localization.md 4.2 | Defined |
| `ui.shop.buy_confirm` | theme-localization.md 4.2 | Defined |
| `ui.quest.abandon_confirm` | theme-localization.md 4.2 | Defined |
| `ui.settings.show_grid` | theme-localization.md 4.2 | Defined |
| `ui.notification.item_acquired` | theme-localization.md 4.2 | Defined |

### 2.2 Gap: BattleScreen Text (Critical for P0)

`screens.md` 2.3 documents BattleScreen with hardcoded text strings. The following text lacks LocalizationKeys and must be defined before the Screen Spec can be finalized:

| Current Hardcoded Text | Screen Region | Proposed LocalizationKey | Priority |
|----------------------|---------------|-------------------------|----------|
| "Turn: {n}" | TurnInfoBar | `ui.battle.turn_indicator.text` | P0 |
| "Phase: Player Turn" | TurnInfoBar | `ui.battle.phase.player` (exists) | P0 |
| "Phase: Enemy Turn" | TurnInfoBar | `ui.battle.phase.enemy` | P0 |
| "Phase: Victory" | TurnInfoBar | `ui.battle.phase.victory` | P0 |
| "Phase: Defeat" | TurnInfoBar | `ui.battle.phase.defeat` | P0 |
| "Attack" | ActionMenu | `ui.battle.action.attack` | P0 |
| "Defend" | ActionMenu | `ui.battle.action.defend` | P0 |
| "Skill" | ActionMenu | `ui.battle.action.skill` | P0 |
| "Item" | ActionMenu | `ui.battle.action.item` | P0 |
| "Wait" | ActionMenu | `ui.battle.action.wait` | P0 |
| "HP" label | CharacterCard | `ui.battle.hp_label` | P0 |
| "MP" label | CharacterCard | `ui.battle.mp_label` | P0 |
| "AP" label | CharacterCard | `ui.battle.ap_label` | P0 |
| "Lv." prefix | CharacterCard | `ui.battle.level_prefix` | P0 |

Note: `phase_key` is already documented as `"ui.battle.phase.player"` in `projection-viewmodel.md` 7, but the existing BattleHudVm uses `&'static str` typed as `"ui.battle.phase.player"` directly, not a named constant from `generated/keys.rs`. This will need alignment when the key generation system is implemented.

### 2.3 Gap: MainMenuScreen Text (P0)

`screens.md` 3.3 lists hardcoded text:

| Current Text | Proposed LocalizationKey | Priority |
|-------------|-------------------------|----------|
| "Fre" (title) | `ui.main_menu.title` | P0 |
| "A Bevy SRPG" (subtitle) | `ui.main_menu.subtitle` | P0 |
| "New Game" | `ui.main_menu.new_game` | P0 |
| "Load Game" | `ui.main_menu.load_game` | P0 |
| "Settings" | `ui.main_menu.settings` | P0 |
| "v0.1.0" (version) | `ui.main_menu.version` | P0 |

Note: `ui.menu.settings` and `ui.menu.quit` already exist in `localization_schema.md` 3.6. The proposed `ui.main_menu.*` namespace is consistent with the existing convention.

### 2.4 Gap: InventoryScreen Text (P1)

| Current Text | Proposed LocalizationKey | Priority |
|-------------|-------------------------|----------|
| "Inventory" | `ui.inventory.title` | P1 |
| "Gold: {n}" | `ui.inventory.gold_display` | P1 |
| "Close" | `ui.inventory.close` | P1 |

### 2.5 Gap: SettingsScreen Text (P1)

| Current Text | Proposed LocalizationKey | Priority |
|-------------|-------------------------|----------|
| "Show Damage Numbers" | `ui.settings.show_damage_numbers` | P1 |
| "Show Minimap" | `ui.settings.show_minimap` | P1 |
| "Show Grid" | `ui.settings.show_grid` (exists) | P1 |
| "Auto Battle" | `ui.settings.auto_battle` | P1 |
| "Theme" | `ui.settings.theme_label` | P1 |
| "Language" | `ui.settings.language_label` | P1 |
| "Master Volume" | `ui.settings.master_volume` | P1 |
| "BGM Volume" | `ui.settings.bgm_volume` | P1 |
| "SFX Volume" | `ui.settings.sfx_volume` | P1 |
| "Battle Speed" | `ui.settings.battle_speed` | P1 |
| "Tooltip Delay" | `ui.settings.tooltip_delay` | P1 |
| "Reset to Defaults" | `ui.settings.reset_defaults` | P1 |

### 2.6 Gap: ShopScreen Text (P1)

| Current Text | Proposed LocalizationKey | Priority |
|-------------|-------------------------|----------|
| Shop name | `ui.shop.greeting` (wider scope) | P1 |
| "Buy" tab | `ui.shop.tab_buy` | P1 |
| "Sell" tab | `ui.shop.tab_sell` | P1 |
| "Cart: {count} items, {total}" | `ui.shop.cart_summary` | P1 |
| "Buy" button | `ui.shop.buy` | P1 |
| "Cancel" | `ui.shop.cancel` | P1 |
| "Confirm" | `ui.shop.confirm` | P1 |

### 2.7 Naming Convention Consistency

The existing convention (`theme-localization.md` 4.2) uses `ui.<scope>.<id>` where scope is a functional domain name (battle, inventory, shop, etc.). The plan proposes a more granular `ui.battle.{screen}.{widget}.{field}` pattern.

**Conflict**: The plan uses `battle_phase_label` for a widget_id that displays a value resolved from `"ui.battle.phase.player"`. The screen-level naming (`battle_<element>`) is for widget IDs, while the LocalizationKey follows the existing `ui.<scope>.<id>` pattern.

**Recommendation**: Keep the existing `ui.<scope>.<id>` pattern for LocalizationKeys (unchanged). Widget IDs use `{screen}_{region}_{element}` snake_case. These are two independent naming systems:
- LocalizationKey (`ui.battle.end_turn`) = text lookup key in .ftl files
- widget_id (`battle_end_turn_btn`) = permanent widget instance identifier

No change to the existing LocalizationKey convention is required.

---

## 3. Data Flow Path Review

### 3.1 Existing Path

The path defined in `projection-viewmodel.md` 8.1 is:

```
Content (assets/config/*.ron)
    ↓ AssetServer loading
DefRegistry (Resource)
    ↓ Projection query
ViewModel (UiStore)
    ↓ Dirty<T> flag
Widget
```

This is the correct and current architecture. No modifications are needed to accommodate Screen Specs.

### 3.2 How widget_id fits into the existing flow

```
07-specs/references/widget-id-map.md (documentation only)
    │  Maps: widget_id → UiBinding variant
    │        widget_id → DefRegistry type (Category A)
    │        widget_id → ViewModel field
    ▼
Widget spawn code (src/ui/screens/*.rs)
    │  Uses widget_id as the UiBinding + entity naming convention
    ▼
Dirty<T> + UiBinding → Widget refresh at runtime
```

The widget_id is a documentation-layer concept. It does NOT introduce a new runtime component, a new system, or a new data structure. It maps to:
- A `UiBinding` variant (existing, defined in `focus-binding.md` 4.2)
- A ViewModel field (existing, defined in `projection-viewmodel.md` 3.4)
- Optionally, a `DefRegistry<T>` type (existing, defined in `03-content/README.md` 5)

### 3.3 Projection Adjustment for Screen Spec

The plan's Event Contract section (3.8) introduces structured `Projection` annotations in the Screen Spec. This is documentation, not a runtime change:

```yaml
TurnStarted:
  source: Domain Event (Combat Domain)
  projection: BattleProjection.project_turn()
  vm_update: BattleHudVm.turn_number += 1
  vm_update: BattleHudVm.phase_key = "ui.battle.phase.player"
  side_effect: mark_dirty::<BattleHudVm>()
```

**Compatibility verdict**: Fully compatible. The existing `projection-viewmodel.md` 7 already documents Projection mappings in tabular form. The Screen Spec YAML format is a different representation of the same information.

### 3.4 Regional State Mapping Impact

The plan introduces per-region state mapping (Loading/Empty/Error states, see plan 1.1 table). This is entirely a @presentation-architect concern -- the Content Platform is not involved because:

- Loading state is determined by data availability (not Def loading)
- Empty state depends on filtered ViewModel content (not Def existence)
- Error state is for network/validation errors (not schema errors)

The Content Platform already has its own error handling in the validation pipeline (8 validation rules, `content-platform-manifesto.md`). Screen-level error states are orthogonal.

### 3.5 Def ↔ UiBinding ↔ ViewModel Field Mapping Table

For completeness of the Screen Spec, the following cross-reference should be included in `widget-id-map.md`:

```yaml
# Widget ID → UiBinding → ViewModel Field → Def Registry
# This table bridges widget documentation with the Content Platform

BattleScreen:
  battle_hp_bar:
    uibinding: UiBinding::Hp
    vm_field: BattleHudVm.hp / max_hp
    def_registry: ~ (runtime instance data)
  battle_mp_bar:
    uibinding: UiBinding::Mp
    vm_field: BattleHudVm.mp / max_mp
    def_registry: ~
  battle_buff_icons_0:
    uibinding: UiBinding::BuffSlot(0)
    vm_field: BattleHudVm.buffs[0]
    def_registry: DefRegistry<BuffDef>
  battle_skill_slot_0:
    uibinding: UiBinding::SkillSlot(0)
    vm_field: SkillPanelVm.skills[0]
    def_registry: DefRegistry<SpellDef>
  battle_character_name:
    uibinding: UiBinding::Name
    vm_field: CharacterPanelVm.name_key
    def_registry: DefRegistry<CharacterDef>
```

This table serves as the formal mapping between the Content Platform (Def types), the UI Binding system (UiBinding enum), and the Screen Spec (widget IDs).

---

## 4. Widget ID Naming Convention Recommendations

### 4.1 Format

The plan already proposes `snake_case` which is consistent with Rust variable naming. This is confirmed as the correct choice.

**Recommendation**: Adopt `{screen}_{region}_{element}` as the base format with the following refinements:

```
Format: {screen}_{region}_{element}_{variant}

screen    = Functional screen name (battle, main_menu, inventory, shop, settings, save_load)
region    = Horizontal/vertical region or functional section (top_bar, action_menu, char_panel)
element   = Specific widget function (hp_bar, end_turn_btn, title_text, item_grid)
variant   = Optional suffix for numbered instances (_0, _1, _2) or sub-variants (_icon, _label)

Examples:
battle_top_bar_turn_indicator    # Turn number display in top bar
battle_action_menu_skill_btn_0   # First skill button in action menu
inventory_main_grid_item_slot_3  # Fourth item slot in inventory grid
settings_graphics_theme_selector # Theme dropdown in graphics tab settings
save_load_list_slot_0            # First save slot in save/load list
```

### 4.2 Max Length Guideline

With 5+ years and 10,000+ assets in mind, widget IDs should not exceed 60 characters. This ensures they fit in typical tooling displays and debug overlays.

Avoid overly deep nesting: `battle_screen_char_panel_buff_section_buff_icon_0` (56 chars, borderline) should be shortened to `battle_buff_icon_0` (19 chars).

### 4.3 Stability Contract

The plan's constitution amendment (Widget ID stable, P0) correctly states: widget_id is permanent. Once assigned, it cannot be renamed, only deprecated.

**Content Architecture implication**: If a widget_id maps to a Def Registry type (Category A), and that Def type is renamed or deprecated, the `widget-id-map.md` must record the deprecation chain:

```yaml
# Deprecated widget IDs (never reassigned)
battle_old_spell_icon:
  status: deprecated
  replaced_by: battle_skill_slot_0
  deprecation_reason: "Spell → Skill terminology alignment, ADR-0XX"

battle_old_mp_bar:
  status: deprecated
  replaced_by: battle_mp_bar
  deprecation_reason: "Renamed for consistency with hp_bar"
```

This deprecation tracking is essential for mod compatibility: a mod referencing `battle_old_spell_icon` should still compile (with a deprecation warning), not break silently.

### 4.4 Existing UiBinding Naming Alignment

The existing `UiBinding` enum uses PascalCase (`Hp`, `Mp`, `SkillSlot(u8)`). Widget IDs use snake_case (`battle_hp_bar`, `battle_skill_slot_0`). This is correct -- they serve different purposes:

- `UiBinding::Hp` = ECS component identifier, PascalCase per Rust convention
- `battle_hp_bar` = design documentation identifier, snake_case per UI spec convention
- Mapping: `battle_hp_bar` ↔ `UiBinding::Hp` in `widget-id-map.md`

No naming conflict exists.

---

## 5. Compatibility Verdict

### 5.1 Fully Compatible (No Change Needed)

| Dimension | Reason |
|-----------|--------|
| Def → Projection → ViewModel data flow | Existing flow (`projection-viewmodel.md` 8.1) directly supports all Screen Spec needs |
| Widget ID ↔ UiBinding mapping | UiBinding enum already covers all BattleScreen and MainMenuScreen widget types |
| Category B & C widget IDs | Pure UI containers and command-emitting IDs have no Def dependency |
| Per-region state mapping | Orthogonal to Content Platform (pure ViewModel concern) |
| Event Contract documentation | YAML format in Screen Spec is documentation only, matches existing Projection mappings |
| LocalizationKey format | Existing `ui.<scope>.<id>` pattern is maintained; widget IDs are independent naming space |

### 5.2 Needs Supplement (Documentation Only)

| Item | What to Add | Where | Priority |
|------|------------|-------|----------|
| Def Registry column in widget-id-map | For Category A widget IDs, add `def_registry` field | `07-specs/references/widget-id-map.md` | P0 |
| UiBinding ↔ ViewModel field mapping | Cross-reference table showing pipeline path | `07-specs/references/widget-id-map.md` | P1 |
| Def dependency table for Screen Specs | When a Screen Spec mentions a widget that displays Def data, annotate the Def type | Each Screen Spec's Event Contract section | P1 |
| Deprecation tracking for widget IDs | Format for recording deprecated/replaced widget IDs | `07-specs/references/widget-id-map.md` | P2 |

### 5.3 Needs Creation (Content Platform Effort)

| Item | What | Where | Priority |
|------|------|-------|----------|
| Missing LocalizationKeys | 15+ keys for BattleScreen text, 5+ for MainMenuScreen | `assets/localization/en-US/ui.ftl` (or equivalent) | **P0** |
| UiTextKey enum alignment | Ensure generated keys module (`generated/keys.rs`) covers all UI text keys | `src/infra/localization/generated/keys.rs` | P0 (alongside localization implementation) |

### 5.4 Summary

```
Content Platform Architecture for Screen Spec:  ████████████████████████ 95% compatible
                                                 ████                   5% needs documentation supplement
                                                 ▏                      1% needs new LocalizationKeys
```

The Content Platform requires **no architectural changes, no Schema changes, no Registry changes, no Pipeline changes** to support the Screen Spec refactoring. The work items identified above are:
- Documentation additions to `widget-id-map.md` (mapping Category A widget IDs to Def types)
- LocalizationKey definitions for text that is currently hardcoded in screen implementations

These are both forward-looking supplements to align existing documentation with the new Screen Spec format, not remediation of existing architecture flaws.

---

*Report by @content-architect. Generated for Phase 1 of the UI Screen Specification refactoring (see `docs/11-refactor/ui-screen-spec-execution-plan.md`).*
