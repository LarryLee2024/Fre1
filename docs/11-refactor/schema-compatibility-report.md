---
id: 11-refactor.schema-compatibility-report
title: Data Schema Compatibility Report for Screen Spec
status: draft
created: 2026-06-22
owner: data-architect
tags:
  - schema-review
  - ui
  - screen-spec
  - data-architect
---

# Data Schema Compatibility Report for Screen Spec

## Sources Consulted

| Source | Authority | Notes |
|--------|-----------|-------|
| `src/ui/view_models/battle_hud.rs` | **Ground truth** | MVP implementation (3 ViewModels) |
| `src/ui/view_models/character_panel.rs` | **Ground truth** | |
| `src/ui/view_models/skill_panel.rs` | **Ground truth** | |
| `src/ui/binding/ui_binding.rs` | **Ground truth** | UiBinding enum |
| `src/ui/binding/dirty_flag.rs` | **Ground truth** | Dirty<T> implementation |
| `src/ui/view_models/mod.rs` | **Ground truth** | UiStore (3 fields) |
| `docs/04-data/capabilities/ui-presentation-schema.md` | **Schema doc (draft)** | Full design, diverges from code |
| `docs/06-ui/04-data-flow/projection-viewmodel.md` | **Code-aligned** | Matches code, reference view |
| `docs/06-ui/02-design-system/widget-composites.md` | **Code-aligned** | Composite widget definitions |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` | **Plan** | Screen Spec structure + widget-id-map |

---

## 1. ViewModel Type Consistency

### 1.1 Critical Finding: Three Divergent Definitions of BattleHudVm

There exist three incompatible versions of `BattleHudVm`. The Screen Spec template's Event Contract references fields from the **code-aligned doc**, but the **schema doc** has a different structure entirely.

| Field | Actual Code | projection-viewmodel.md | ui-presentation-schema.md | Used by Screen Spec? |
|-------|-------------|------------------------|--------------------------|---------------------|
| `hp` | `f32` | `f32` | `u32` | Yes — Event Contract |
| `max_hp` | `f32` | `f32` | `u32` | Implicit |
| `mp` | `f32` | `f32` | `u32` | Implicit |
| `max_mp` | `f32` | `f32` | `u32` | Implicit |
| `ap` | `f32` | `f32` | **MISSING** (uses `action_points: u32`) | Yes — BattleScreen TopBar |
| `max_ap` | `f32` | `f32` | **MISSING** (uses `max_action_points: u32`) | Implicit |
| `turn_number` | `u32` | `u32` | **MISSING** (uses `current_turn: u32`) | Yes — Event Contract |
| `phase_key` | `&'static str` | `&'static str` | **MISSING** (uses `phase: BattlePhaseVm`) | Yes — Event Contract |
| `active_character` | **MISSING** | **MISSING** | `Option<CharacterId>` | No |
| `cooldowns` | **MISSING** | **MISSING** | `HashMap<SkillId, f32>` | No |

**Verdict**: Three incompatible BattleHudVm definitions across the codebase. ui-presentation-schema.md §4.2 is a **design document that does not match the real code**, and its `active_character`/`cooldowns` fields are not yet implemented. The Screen Spec is safe if it follows the actual code, but the schema doc must be reconciled.

### 1.2 Screen Spec Event Contract Field Verification

The execution plan's Event Contract (§3.8) uses these field references:

```yaml
vm_update: BattleHudVm.hp ← damage_value
vm_update: BattleHudVm.turn_number += 1
vm_update: BattleHudVm.phase_key = "ui.battle.phase.player"
```

All three fields (**hp**, **turn_number**, **phase_key**) exist in the actual code. The references are **compatible** with the current implementation.

However, `damage_value` is an undefined variable — the Screen Spec template should specify the data source explicitly (e.g., `DamageApplied.damage_amount` with type `f32`).

### 1.3 CharacterPanelVm Verification

| Field | Actual Code | Screen Spec Reference | Compatible? |
|-------|-------------|----------------------|------------|
| `character_id: u32` | Yes | Implicit (CharacterCard) | Yes |
| `name_key: &'static str` | Yes | Implicit | Yes |
| `level: u32` | Yes | Implicit (CharacterCard) | Yes |
| `hp: f32` | Yes | Implicit (HP bar) | Yes |
| `max_hp: f32` | Yes | Implicit | Yes |
| `mp: f32` | Yes | Implicit (MP bar) | Yes |
| `max_mp: f32` | Yes | Implicit | Yes |

The BattleScreen Widget Tree references `CharacterCard` and `CharacterStatusPanel`. The current `CharacterPanelVm` does NOT have enough fields for `CharacterStatusPanel` which requires `buffs: Vec<BuffVm>` and `ap_current/ap_max` (see widget-composites.md §3.2 Props). This is a **gap** — the composite widget's Props exceed the current ViewModel's fields.

### 1.4 SkillPanelVm Verification

| Field | Actual Code | Screen Spec Reference | Compatible? |
|-------|-------------|----------------------|------------|
| `skills: HashMap<u32, SkillSlotVm>` | Yes | Widget Contract references SkillPanel | Yes |
| `selected: Option<SkillId>` | **MISSING** | Implicit (skill selection) | **Gap** |
| `ap_remaining: u32` | **MISSING** | Implicit (AP cost checking) | **Gap** |
| `max_ap: u32` | **MISSING** | Implicit | **Gap** |

SkillPanelVm in code is a thin wrapper around `HashMap<u32, SkillSlotVm>` with no `selected` or `ap_remaining` fields, but the widget-composites.md SkillPanel Props and SkillSlot Contract both require these fields for interaction validation.

### 1.5 ViewModels Referenced by Screen Spec Widgets but NOT Defined

The Screen Spec will define Widget Trees that reference composite widgets. These widgets have Props that require ViewModels which do not currently exist in code:

| Composite Widget | Required ViewModel | Exists? | Referenced In |
|-----------------|-------------------|---------|---------------|
| CharacterPortrait (Molecule) | CharacterPortraitVm | **No** | widget-composites.md §2.2 |
| CharacterStatusPanel (Organism) | CharacterStatusPanelVm | **No** | widget-composites.md §3.2 |
| TurnOrderBar (Organism) | TurnOrderBarVm | **No** | widget-composites.md §3.4 |
| TurnIndicator (Molecule) | TurnIndicatorVm | **No** | widget-composites.md §2.8 |
| InventoryGrid (Organism) | InventoryGridVm | **No** | widget-composites.md §3.5 |
| QuestEntry (Molecule) | QuestEntryVm | **No** | widget-composites.md §2.4 |
| QuestLog (Organism) | QuestLogVm | **Schema only** | widget-composites.md §3.6 |
| DialogueChoice (Molecule) | DialogueChoiceVm | **No** | widget-composites.md §2.5 |
| DialoguePanel (Organism) | DialoguePanelVm | **No** | widget-composites.md §3.7 |
| ShopItemCard (Molecule) | ShopItemCardVm | **No** | widget-composites.md §2.6 |
| ShopPanel (Organism) | ShopPanelVm (ShopVm exists in schema only) | **Schema only** | widget-composites.md §3.8 |
| BuffIcon (Molecule) | BuffVm | **Schema only** | widget-composites.md §2.7 |

**Impact**: Screen Specs for BattleScreen, InventoryScreen, ShopScreen, QuestLogScreen, and SettingsScreen will reference widgets whose ViewModels don't exist as implementable code structs. The Specs can still be written (they describe layout, not implementation), but a ViewModel definition phase must precede implementation.

---

## 2. UiBinding Enum Completeness

### 2.1 Ground Truth vs Documentation Mismatch

The actual code (`src/ui/binding/ui_binding.rs`) has the most complete UiBinding:

```rust
pub enum UiBinding {
    // Battle HUD: Hp, MaxHp, Mp, MaxMp, Ap, MaxAp, Turn, Phase
    // Character Panel: Level, Exp, Name, CharacterLevel
    // Skill Panel: SkillSlot(u8), Cooldown
    // Inventory: ItemSlot(u8), Gold
    // Quest: QuestEntry(u16)
    // General: Tooltip, Modal, Notification, Text, Icon
}
```

The schema doc (`ui-presentation-schema.md §23`) is **missing** `CharacterLevel`, `Text`, and `Icon` — these exist in code but not in the data schema document.

### 2.2 Screen Spec widget-id-map Compatibility

The widget-id-map in the execution plan (§3.9) maps widget_ids to UiBinding variants:

| widget_id | Mapped Binding | Exists? | Issue |
|-----------|---------------|---------|-------|
| `turn_indicator` | `UiBinding::Turn` | Yes | OK |
| `phase_label` | `UiBinding::Phase` | Yes | OK |
| `hp_bar` | `UiBinding::Hp` | Yes | OK |
| `mp_bar` | `UiBinding::Mp` | Yes | OK |
| `title_text` | `UiBinding::Text` | Yes (code) | Missing from schema doc |
| `buff_icons_0` | `UiBinding::BuffSlot(0)` | **No** | **Missing variant** |
| `char_panel` | `UiBinding::None` | **No** | **Missing variant** |

**Issue 1 — `UiBinding::BuffSlot(u8)` does not exist.**
There is no BuffSlot variant anywhere in the enum. Buff icons currently use `StatusIcon` widget (widget-atoms.md §11.2) and `BuffVm` ViewModel, but have no UiBinding variant. The widget-id-map needs to either:
- Add `BuffSlot(u8)` to UiBinding (preferred — consistent with `SkillSlot(u8)`/`ItemSlot(u8)` pattern)
- Or map buff icons to the generic `Icon` binding

**Issue 2 — `UiBinding::None` does not exist.**
The map uses `UiBinding::None` extensively for container widgets (root, top_bar, battle_area, char_panel, action_menu). But there is no `None` variant — widgets without a binding simply don't have a `UiBinding` component. The widget-id-map should use `(none)` or `—` rather than `UiBinding::None`.

**Issue 3 — Missing Screen-level bindings.**
The Screen Spec references `end_turn_btn` with `UiBinding::None`. But the screens.md (§2.4) maps EndTurnButton to `BattleAction::EndTurn`. If a UiBinding variant is needed, `UiBinding::EndTurn` could be added to the Battle HUD category. Currently, it's handled by a `BattleAction` component marker. This is acceptable — not every button needs a UiBinding.

### 2.3 Parameterized Variant Adequacy

The current parameterized pattern (`SkillSlot(u8)`, `ItemSlot(u8)`, `QuestEntry(u16)`) is sufficient. No new parameterized patterns are needed from the Screen Spec. The `Chars(u8)` dimension is already covered by per-entity binding.

---

## 3. Dirty<T> Mechanism Compatibility

### 3.1 Current Implementation

The actual code (`src/ui/binding/dirty_flag.rs`) implements `Dirty<T>` as a `Component` on Widget entities:

```rust
pub struct Dirty<T: Reflect + Default + Clone + Send + Sync + 'static> {
    pub inner: T,
    is_dirty: bool,
}
```

Key behaviors:
- `get_mut()` auto-marks dirty
- `consume()` returns true once then clears
- Initial state is dirty (triggers first render)

### 3.2 Screen Spec Event Contract Pattern

The execution plan's Event Contract uses a type-parameterized mark pattern:
```yaml
side_effect: mark_dirty::<BattleHudVm>()
```

This is **conceptually compatible**, but the current implementation doesn't support `mark_dirty::<BattleHudVm>()` as a standalone function. The pattern must be:

```rust
// Projection updates a specific UiStore field, not a generic type
store.battle_hud.get_mut().hp = trigger.new_hp;
// The get_mut() call auto-marks Dirty<BattleHudVm> if that component exists
// on the BattleHud entity.
```

The Event Contract's `mark_dirty::<BattleHudVm>()` is a documentation shorthand. As long as Screen Specs understand the projection pattern (direct field mutation on UiStore), there is no incompatibility.

### 3.3 Per-Region State Mapping Gap

**Significant gap**: The Screen Spec defines per-region state mapping (`Loading/Empty/Normal/Error`) as a requirement (§3.7, §4 Step 4 field 8). The current `Dirty<T>` mechanism only has a binary dirty/clean flag. **There is no mechanism for per-region Loading/Empty/Error state tracking.**

The current ViewModels have no concept of "this region is loading" or "this region is empty." Widgets currently render with default `Default::default()` ViewModel values until a real projection occurs.

**Recommendation**: This needs to be addressed separately. Options include:
- Short-term: Document in Screen Spec that Loading/Empty/Error states are NOT yet supported by the data layer, and current behavior is to render Default ViewModel values.
- Long-term: Add an `OptionalVm<T>` wrapper or `RegionState` enum to ViewModel fields that can be Loading/Empty/Error.

### 3.4 Projection→Dirty Data Path

The existing data path is:
```
Domain Event → Observer → Projection → UiStore.field.get_mut() → auto mark_dirty()
```

This is compatible with all Screen Spec requirements. No modification needed.

---

## 4. UiStore Structure Completeness

### 4.1 Current UiStore (Actual Code)

```rust
pub struct UiStore {
    pub battle_hud: BattleHudVm,       // EXISTS
    pub character_panel: CharacterPanelVm,  // EXISTS
    pub skill_panel: SkillPanelVm,      // EXISTS
}
```

### 4.2 Screen Spec Requirements

The 6 Screen Specs will reference UiStore fields as follows:

| Screen | Needs UiStore Field | Exists? | Priority |
|--------|--------------------|---------|----------|
| BattleScreen | `battle_hud` | Yes | P0 |
| BattleScreen | `skill_panel` | Yes | P0 |
| BattleScreen | `character_panel` | Yes | P0 |
| MainMenuScreen | None | N/A | P0 |
| InventoryScreen | `inventory` | **No** | P1 |
| SettingsScreen | None (reads UiSettings directly) | N/A | P1 |
| ShopScreen | `shop` | **No (schema only)** | P1 |
| SaveLoadScreen | None (reads Save Domain) | N/A | P1 |
| QuestLogScreen | `quest_log` | **No (schema only)** | P1 |

### 4.3 Gap Analysis

| Field | Status | Impact |
|-------|--------|--------|
| `inventory: InventoryVm` | Not in code, defined in schema doc | InventoryScreen Spec can be written but not implemented. InventoryVm must be coded before implementation. |
| `shop: ShopVm` | Not in code, defined in schema doc | ShopScreen Spec can be written (Spec before code is intended), but ShopVm must be coded before implementation. |
| `quest_log: QuestLogVm` | Not in code, defined in schema doc | QuestLogScreen Spec can be written, but QuestLogVm must be coded before implementation. |
| `notification_queue: Vec<NotificationVm>` | NotificationVm exists in code (`src/ui/overlay/notification.rs`), but NOT in UiStore | NotificationVm exists as an overlay component but there's no queue in UiStore. The schema doc's `notification_queue` field is not implemented. |
| `modal_stack: Vec<ModalVm>` | ModalVm exists in schema doc only | Not implemented anywhere. Modal overlay uses ModalService pattern, not UiStore. |

**Key Finding**: The schema doc `ui-presentation-schema.md` §4.1 defines a richer UiStore with 8 fields. The actual code has 3 fields. The Screen Spec plan assumes the full UiStore exists, but it does not.

---

## 5. Compatibility Conclusion

### 5.1 Compatible (No Changes Needed)

- **BattleHudVm fields** used by Screen Spec's Event Contract (`hp`, `turn_number`, `phase_key`) — all exist in actual code
- **CharacterPanelVm structure** — matches Screen Spec's implicit CharacterCard requirements
- **UiBinding core variants** (Hp, Mp, Ap, Turn, Phase, SkillSlot, Level, Exp, Name, Text) — all exist
- **Dirty<T> mechanism** — the Projection → ViewModel → mark_dirty path is compatible
- **UiStore field access pattern** — direct struct field access works for all 3 existing fields
- **Projection→Observer pattern** — fully compatible with Event Contract wiring

### 5.2 Needs Documentation Clarification

| Item | Action |
|------|--------|
| UiBinding `None` usage in widget-id-map | Replace `UiBinding::None` with `(none)` or `—` to avoid implying a non-existent enum variant |
| `mark_dirty::<T>()` generic syntax in Event Contract | Document that the actual pattern is `store.field.get_mut() → auto mark_dirty()`, not a generic function call |
| ViewModel type divergence between code and schema doc (`f32` vs `u32`, missing/enum fields) | Update `ui-presentation-schema.md` to match actual code, or plan a migration if enum-based phase is intended |
| UiBinding `Text`/`Icon`/`CharacterLevel` missing from schema doc | Add missing variants to `ui-presentation-schema.md §23` |

### 5.3 Needs Additive Changes Before Screen Spec Implementation

| Requirement | Priority | Action |
|------------|----------|--------|
| `UiBinding::BuffSlot(u8)` | **P0** (widget-id-map references it) | Add variant to `src/ui/binding/ui_binding.rs` before widget-id-map is finalized |
| `SkillPanelVm.selected: Option<SkillId>` | **P1** (widget-composites §3.1 Props) | Add field to `src/ui/view_models/skill_panel.rs` |
| `SkillPanelVm.ap_remaining: u32` + `max_ap: u32` | **P1** | Add fields for skill availability check |
| `CharacterPanelVm.buffs: Vec<BuffVm>` | **P1** (CharacterStatusPanel needs it) | Either add to existing Vm or create CharacterStatusPanelVm |

### 5.4 Needs New Implementation Before Screen Spec Implementation

| ViewModel | Needed By | Reference |
|-----------|-----------|-----------|
| CharacterStatusPanelVm | BattleScreen CharacterCard/StatusPanel | widget-composites.md §3.2 |
| TurnOrderBarVm + TurnIndicatorVm | BattleScreen TurnOrderBar | widget-composites.md §3.4, §2.8 |
| CharacterPortraitVm | BattleScreen CharacterCard | widget-composites.md §2.2 |
| InventoryVm + InventorySlotVm + filters | InventoryScreen | schema doc §4.5 |
| ShopVm + ShopSlotVm | ShopScreen | schema doc §4.6 |
| QuestLogVm + QuestSlotVm + filters | QuestLogScreen | schema doc §4.7 |
| BuffVm | Multiple widgets | schema doc §4.3 (within CharacterPanelVm) |

### 5.5 Data Schema Document Action Items

| Doc | Issue | Action |
|-----|-------|--------|
| `docs/04-data/capabilities/ui-presentation-schema.md` §4.2 | BattleHudVm uses `u32` + `BattlePhaseVm` + missing ap fields | **Reconcile with code**: code uses `f32` + `&'static str phase_key`. Decide whether to migrate to enum. |
| `docs/04-data/capabilities/ui-presentation-schema.md` §4.3 | CharacterPanelVm has 11 fields, code has 7 | Sync to actual code, move extended fields to CharacterStatusPanelVm |
| `docs/04-data/capabilities/ui-presentation-schema.md` §23 | Missing CharacterLevel, Text, Icon variants | Add missing variants |
| `docs/06-ui/04-data-flow/projection-viewmodel.md` §3.4 | `&'static str` type for phase_key is problematic | `&'static str` cannot be serialized. Should be `UiTextKey` (which is `String`) for future save/replay compatibility. |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` §3.9 | `UiBinding::None` usage | Replace with `(no binding)` notation |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` §3.9 | `UiBinding::BuffSlot(0)` usage | Add BuffSlot variant to UiBinding first |

### 5.6 Risk Summary

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Screen Spec Event Contract references ViewModel fields that don't exist for composite widgets | **High** — Event Contract is P1, but composite widgets are P0 | Phase Screen Spec delivery: write Spec first (P0), then implement missing ViewModels (P0.5), then implement code (P1) |
| Per-region Loading/Empty/Error state has no data mechanism | **Medium** — Screen Spec requires it, data layer doesn't support it | Document as "future: not yet supported" in Spec, use default values in the short term |
| UiStore has only 3 fields, Screen Spec needs 6+ | **Low** — additive field model, UiStore is a single Resource | Add fields one by one as each Screen is implemented |
| ViewModels are `&'static str` for text keys (non-serializable) | **Low** — ViewModels aren't persisted, but this prevents future debugging tooling | Replace with `UiTextKey` (String-based) in a separate refactor |
| Three divergent BattleHudVm definitions | **Medium** — causes confusion for AI code generation | Define actual code as SSOT, mark schema doc as draft needing update |

### 5.7 Overall Recommendation for Screen Spec Phase

**Screen Specs can be written now** without blocking on data layer gaps, because Specs describe layout and contract, not implementation. However, the following data prerequisites must be completed before Specs can be implemented:

1. **Before any Spec implementation**: Add `UiBinding::BuffSlot(u8)` and document that `(no binding)` is the correct notation for containers.
2. **Before BattleScreen implementation**: Ensure CharacterStatusPanelVm, TurnOrderBarVm are defined (even if stub implementations).
3. **Before InventoryScreen/ShopScreen implementation**: InventoryVm, ShopVm, and related filter/sort types must be coded (use the schema doc as blueprint).
4. **Before QuestLogScreen implementation**: QuestLogVm must be coded.
5. **Address the schema doc divergence**: Reconcile `ui-presentation-schema.md` with actual code to ensure AI code generation from docs produces correct implementations.

---

## Appendix: ViewModel Field Cross-Reference Table

### Current Actual Code ViewModels

```
BattleHudVm
  hp: f32
  max_hp: f32
  mp: f32
  max_mp: f32
  ap: f32
  max_ap: f32
  turn_number: u32
  phase_key: &'static str

CharacterPanelVm
  character_id: u32
  name_key: &'static str
  level: u32
  hp: f32
  max_hp: f32
  mp: f32
  max_mp: f32

SkillPanelVm
  skills: HashMap<u32, SkillSlotVm>

SkillSlotVm
  skill_id: u32
  name_key: &'static str
  cooldown_remaining: u32
  max_cooldown: u32
  is_usable: bool
  ap_cost: u32

TooltipVm          (in src/ui/overlay/tooltip.rs)
NotificationVm     (in src/ui/overlay/notification.rs)
```
