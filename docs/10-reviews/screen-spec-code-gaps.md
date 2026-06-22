---
status: active
type: review
reviewer: code-reviewer
created: 2026-06-22
scope: src/ui/
standard: SSPEC (Screen Specification)
---

# SSPEC (Screen Specification) Code Gap Analysis

## Purpose

Evaluate `src/ui/` against the Screen Specification (SSPEC) standard. SSPEC
requires:
1. Factory-constructed Screens -- no direct `commands.spawn` of Node/Button/Text
2. No direct Domain queries -- ViewModel (UiStore) is the sole data source for Widgets
3. `LocalizationKey` for all user-visible text -- no hardcoded strings
4. `StyleToken` (Theme) for all colors/fonts/spacing -- no hardcoded pixels or rgba
5. Deterministic despawn on lifecycle exit
6. Composable Screen + Overlay hierarchy

Severity: P0 = must fix (architecture violation), P1 = should fix (code quality/technical debt), P2 = observable (minor, track for later)

---

## 1. Screen Construction -- Factory Pattern Compliance

### 1.1 Primitives Factory Coverage (Good Foundation)

The primitives layer provides well-formed factory functions covering all basic
UI atoms. All use `spawn_*` naming, accept `Theme` via parameter, and return
`Entity`. Covered atoms:

| Factory | Type | Localized Variant |
|---------|------|-------------------|
| `spawn_text` | `Text` | `spawn_localized_text` |
| `spawn_button` | `Button` | `spawn_localized_button` |
| `spawn_panel` | `Panel` | -- |
| `spawn_progress_bar` | `ProgressBar` | -- |
| `spawn_toggle` | `Toggle` | built-in key support |
| `spawn_modal` | `Modal` | built-in key support |
| `spawn_list` | `List` | -- |
| `spawn_tab_panel` | `TabPanel` | MISSING |
| `spawn_scroll_panel` | `ScrollPanel` | -- |
| `spawn_select_list` | `SelectList` | -- |

### 1.2 Screen Factory Analysis

| Screen | Factory Function | Violations | Severity |
|--------|-----------------|-----------|----------|
| MainMenu | `spawn_main_menu` | Overrides factory Node after `spawn_panel` call | P2 |
| Battle | `spawn_battle_screen` | **Raw `commands.spawn((Node{...}, BattleScreen))` at line 53** | **P0** |
| Settings | `spawn_settings_screen` | Overrides factory Node after `spawn_panel` call | P2 |
| SaveLoad | inner factory | Uses `spawn_panel` then raw Node override; hardcoded slots | P2 |
| Shop | `spawn_shop_panel` | Overrides factory Node at line 48 with raw layout | P2 |
| Inventory | `spawn_inventory_screen` | Uses `spawn_panel` then raw Node override | P2 |

**Details:**

- **P0 -- `src/ui/screens/battle/mod.rs:53-63`**: The BattleScreen root is spawned via
  raw `commands.spawn((Node{...}, BattleScreen, Name {...}))` instead of calling
  `spawn_panel`. This is the only screen that completely bypasses the primitives
  factory for its root container.

- **P2 -- All screens**: Every screen factory calls `spawn_panel` then immediately
  overrides the returned entity's Node component:
  ```rust
  let root = spawn_panel(commands, theme, PanelVariant::Basic);
  commands.entity(root).insert((Node { width: 100%, ... }));
  ```
  This pattern is a code smell -- the factory return value is discarded. A future
  SSPEC refinement should provide `spawn_fullscreen_panel` or similar.

### 1.3 Widget Factory Analysis

| Widget | Factory | ViewModel Binding | Severity |
|--------|---------|-------------------|----------|
| CharacterCard | `spawn_character_card` | `Dirty<CharacterPanelVm>` (wired) | OK |
| SkillSlot | `spawn_skill_slot` | `Dirty<SkillPanelVm>` (wired) | OK |
| ActionMenu | `spawn_action_menu` | NO Dirty binding | **P1** |
| ShopPanel | `spawn_shop_panel` | NO Dirty binding | **P1** |
| InventoryGrid | `spawn_inventory_grid` | NO Dirty binding | **P1** |
| ShopItemCard | `spawn_shop_item_card` | NO Dirty binding | P2 (child of ShopPanel) |
| InventoryItemRow | `spawn_inventory_item_row` | NO Dirty binding | P2 (child of InventoryGrid) |

**P1 -- `src/ui/widgets/action_menu/factory.rs`**: ActionMenu does not carry
`Dirty<ActionMenuVm>` (which does not exist). It relies on static construction
with hardcoded strings. No ViewModel drives its state.

---

## 2. ViewModel Usage Violations

### 2.1 Architecture Violations (P0)

Direct Domain queries in UI code -- these break the ViewModel fire wall:

1. **P0 -- `src/ui/screens/battle/systems.rs:30`**:
   ```rust
   turn_queue: Option<Res<TurnQueue>>,
   ```
   `TurnQueue` is a `core::domains::combat` type. The UI observer directly imports
   and queries a domain component. Rule violated: `docs/06-ui/04-data-flow/`
   mandates UiStore as sole data source.

   Impact: On `BattleAction::EndTurn`, the unit_id is extracted from
   `turn_queue.current()` at lines 42-46, which makes replay-dependent UI state
   non-deterministic from UI perspective.

2. **P0 -- `src/ui/screens/battle/visibility.rs:20`**:
   ```rust
   fn battle_zone_visibility_system(
       battle_phase: Res<State<BattlePhase>>,
   ```
   `BattlePhase` is a domain state. Zone visibility should be driven by
   `BattleHudVm.phase_key` or a dedicated visibility ViewModel.

3. **P1 -- `src/ui/screens/battle/visibility.rs`**: Uses `Changed<BattleZone>` and
   `State<BattlePhase>` directly. No ViewModel projection exists for zone
   visibility.

4. **P1 -- `src/ui/screens/battle/systems.rs:47`**:
   ```rust
   .unwrap_or_default()
   ```
   In business code. This masks logic errors as empty strings.

### 2.2 ViewModel Definition Gaps

| ViewModel | Defined | Consumed | Projection Wired |
|-----------|---------|----------|-----------------|
| `BattleHudVm` | Yes (7 fields) | NO | Partially (direct Query in projections) |
| `CharacterPanelVm` | Yes | CharacterCard via Dirty | Hardcoded default values |
| `SkillPanelVm` | Yes | SkillSlot via Dirty | Projection exists (hardcoded data) |

**P1 -- BattleHudVm is defined but no Widget consumes it**. The struct has
`hp`, `max_hp`, `mp`, `max_mp`, `turn_number`, `phase_key` but no widget
system reads `Dirty<BattleHudVm>`. The turn indicator text at
`battle/mod.rs:73` is a hardcoded string, not bound to any VM.

### 2.3 Projection Layer Issues

**P1 -- `src/ui/projections/battle.rs:141`**:
```rust
fn on_turn_started_projection(... query: Query<&ActionPoints>)
```
Projections are supposed to be pure functions that translate domain events to
ViewModel updates. Queries for `ActionPoints` inside a projection blur the
boundary. Comment notes "intentional bridge" but this undermines the
ViewModel isolation principle.

**P1 -- `src/ui/projections/battle.rs:231-270`**:
```rust
fn on_character_panel_projection(... query: Query<&Name>)
```
Similarly queries domain component `Name` directly in projection.

---

## 3. Hardcoded Text

### 3.1 P0 -- User-Visible Hardcoded Strings

These are user-visible text that must use `LocalizationKey`:

| Location | String | Context |
|----------|--------|---------|
| `screens/main_menu/mod.rs:78` | `"Fre"` | Game title |
| `screens/main_menu/mod.rs:102` | `"A Bevy SRPG"` | Subtitle |
| `screens/main_menu/mod.rs:151` | `"v0.1.0"` | Version text |
| `screens/main_menu/mod.rs:122` | `"New Game"` | Button fallback (key exists: loc::ui::NEW_GAME) |
| `screens/main_menu/mod.rs:129` | `"Load Game"` | Button fallback (key exists: loc::ui::LOAD_GAME) |
| `screens/main_menu/mod.rs:136` | `"Settings"` | Button fallback (key exists: loc::ui::SETTINGS) |
| `screens/battle/mod.rs:73` | `"Turn: 3    Phase: Player Turn"` | Turn indicator -- completely hardcoded debug text |
| `screens/battle/mod.rs:95` | `"Aria"` | Character name |
| `screens/battle/mod.rs:118` | `"End Turn"` | Button fallback (key exists: loc::ui::BATTLE_END_TURN) |
| `screens/save_load/mod.rs:111` | `"Save/Load"` | Screen title |
| `screens/save_load/mod.rs:134` | `"Save Slot {}"` | Slot format |
| `screens/save_load/mod.rs:138` | `"Empty"` | Empty slot label |
| `screens/save_load/mod.rs:153,165` | `"Save"` / `"Load"` | Button fallbacks |
| `screens/settings/mod.rs:129` | `"Show Damage Numbers"` | Toggle label |
| `screens/settings/mod.rs:147` | `"Dark Theme"` | Toggle label |
| `screens/settings/mod.rs:163` | `"Close"` | Button fallback |
| `screens/settings/mod.rs:180` | `"Save"` | Button fallback |
| `screens/inventory/systems.rs:33,39` | `"player"` | Hardcoded user ID |

### 3.2 P1 -- Widget-Level Hardcoded Strings

| Location | String | Context |
|----------|--------|---------|
| `widgets/action_menu/factory.rs:47-71` | `"Attack"`, `"Defend"`, `"Skill"`, `"Item"`, `"Wait"` | Action labels (keys exist) |
| `widgets/character_card/factory.rs:73` | `format!("Lv.{}", level)` | Level format |
| `widgets/shop_panel/factory.rs:76` | `"Shop"` | Shop title |
| `widgets/shop_panel/factory.rs:84` | `"Gold: 999"` | Gold display |
| `widgets/shop_panel/factory.rs:95` | `"Buy"`, `"Sell"` | Tab labels |
| `widgets/shop_panel/factory.rs:100-102` | `"Health Potion"`, `"Mana Potion"`, `"Antidote"` | Item names |
| `widgets/shop_panel/factory.rs:119` | `"Old Sword"`, `"Leather Armor"` | Sell item names |
| `widgets/shop_item_card/factory.rs:52` | `format!("Gold: {}", price)` | Price format |
| `widgets/shop_item_card/factory.rs:53` | `format!("Stock: {}", stock)` | Stock format |
| `widgets/inventory_item_row/factory.rs` | `format!("x{}", qty)` | Quantity format |
| `widgets/inventory_grid/factory.rs` | `"Inventory"`, `"Gold: 100"`, item names | Full inventory text |
| `primitives/progress_bar/factory.rs:63-67` | `"HP "`, `"MP "`, `"XP "` | Progress bar prefixes |

### 3.3 P2 -- Non-User-Facing Hardcoded Strings

| Location | String | Context |
|----------|--------|---------|
| `widgets/shop_panel/factory.rs:149` | `"Sell"` | Button fallback (key exists: loc::economy::SHOP_SELL_TEXT) |
| `widgets/shop_item_card/factory.rs:116` | `"Buy"` | Button fallback (key exists: loc::economy::SHOP_BUY_TEXT) |

Note: These are downgraded to P2 because the `spawn_localized_button` calls
DO pass the correct localization key -- the hardcoded string is only the
fallback. This is the correct pattern per SSPEC.

---

## 4. Color/Font Violations -- Hardcoded vs StyleToken

### 4.1 Good: Theme Token Usage

The theme system (`src/ui/theme/`) is well designed:

- `UiColors` with `dark()`/`light()` constructors covering ~20 semantic tokens
- `UiSpacing` with named tokens (xs=4, sm=8, md=16, lg=24, xl=32, xxl=48, border_radius_sm/lg, button_height)
- `UiTypography` with font paths and size/weight tokens

All primitives factories correctly consume Theme:
- `button/factory.rs`: `theme.colors.accent_*`, `theme.colors.surface_*`, `theme.colors.text_*`
- `panel/factory.rs`: `theme.colors.surface_*`, `theme.colors.border_*`
- `text/factory.rs`: `theme.colors.text_*`, `theme.typography.*`
- `progress_bar/factory.rs`: `theme.colors.feedback_*`, `theme.colors.accent_*`
- `toggle/factory.rs`: `theme.colors.accent_*`, `theme.colors.surface_*`

### 4.2 P1 Violations

1. **P1 -- `src/ui/screens/main_menu/mod.rs:87`**:
   ```rust
   font_size: FontSize::Px(48.0),
   ```
   Hardcoded font size for game title. The theme defines `size_display: 36.0`
   and `size_title: 24.0` but neither is used. The title uses `TextVariant::Title`
   (which gives default 24px via `font_size_for_variant`) then overrides with
   hardcoded `48.0`. Either a new `TextVariant::Display` is needed or the theme
   should expose `size_display`.

2. **P1 -- `src/ui/primitives/modal/factory.rs:61`**:
   ```rust
   let overlay_color = Color::srgba(0.0, 0.0, 0.0, 0.6);
   ```
   Modal overlay alpha is hardcoded. Should be `theme.colors.overlay` or similar.

### 4.3 P2 Observations

- `Color::NONE` is used in several places for BorderColor. This is acceptable
  but a future Theme token for `border_none` would be consistent.
- `button/factory.rs:83-86`: The Secondary variant uses `Color::NONE` border if
  not Secondary. This pattern is acceptable but slightly opaque.

---

## 5. Screen Composition & Layout

### 5.1 BattleScreen 9-Zone Layout

**Good**: The zone layout (`battle/layout.rs`) uses absolute positioning with
`spawn_zone` factory and `BattleZone` enum (9 zones). The zone factory
correctly reads `theme.spacing` tokens for padding/margins.

**Issues**:

- **P1 -- `battle/layout.rs:53-54`**: Z2 TopCenter has a TODO for missing
  horizontal centering:
  ```rust
  // TODO[P2] missing: center horizontally
  ```

- **P2 -- `battle/mod.rs:79,85,126`**: Z2, Z3, Z8 are empty (TODO comments).
  These are deliberate P2-scoped placeholders.

### 5.2 Lifecycle Management

| Screen | Creation | Despawn | Score |
|--------|----------|---------|-------|
| MainMenu | `Startup` | `OnExit(GameState::MainMenu)` | OK for initial |
| Battle | `OnEnter(GameState::Combat)` | `OnExit(GameState::Combat)` | OK |
| Settings | Observer on `UiCommand::OpenScreen` | Observer on `UiCommand::CloseScreen` | Fragile |
| SaveLoad | Observer on `UiCommand::OpenScreen` | Observer on `UiCommand::CloseScreen` | Fragile |
| Shop | Observer on `UiCommand::OpenScreen` | Observer on `UiCommand::CloseScreen` | Fragile |

**P1 -- Inconsistent lifecycle pattern**. The comment at `screens/mod.rs:48`
acknowledges:
```rust
// 未来将迁移到 OnEnter(GameState::...) + OnExit(...)
```
Overlay screens use Observer-based creation/despawn which does not integrate
with the ScreenStack navigation system. When `UiScreenState` is wired, these
would need to be migrated.

**P2 -- `src/ui/navigation/screen_state.rs`**: `UiScreenState` is defined with
`ScreenLifecycle` enum but is never inserted as a Resource. The ScreenStack
is also never pushed/popped.

### 5.3 Screen Composition Summary

```
Current:
  Screen
    ├── Panel (overridden)
    ├── Primitive texts (hardcoded or localized-fallback)
    ├── Widgets (some with Dirty binding, some without)
    └── Buttons (localized via spawn_localized_button)

Expected (SSPEC):
  Screen (Factory)
    ├── Panel (spawn_panel, no override)
    ├── Texts (ALL via spawn_localized_text with loc::ui::* keys)
    ├── Widgets (ALL with Dirty<ViewModel> binding)
    ├── Buttons (ALL via spawn_localized_button with loc::* keys)
    └── ViewModel projection wired to Domain Event
```

---

## 6. Localization Key Format Inconsistency

**P1 -- Key format mismatch**. The codebase uses two incompatible key formats:

| Format | Example | Location |
|--------|---------|----------|
| Generated constants (snake_case) | `loc::ui::BATTLE_END_TURN`, `loc::ui::CLOSE` | Most screens |
| Raw dot-notation strings | `"ui.settings.show_damage"`, `"ui.settings.dark_theme"` | Settings toggle spawns |

The settings toggle at `screens/settings/mod.rs:128` passes a raw key string:
```rust
spawn_toggle(commands, theme, "ui.settings.show_damage", "Show Damage Numbers", ...)
```

This bypasses the generated `loc::*` constants. Either `spawn_toggle` should
accept a `&'static str` key (which it does -- the API is correct) and have
callers use `loc::ui::SETTINGS_SHOW_DAMAGE`, or the generated keys module
needs an entry for these.

**P2 -- `spawn_tab_panel` doesn't support keys**. Tab panel at
`widgets/shop_panel/factory.rs:95` notes this explicitly:
```rust
// MVP: uses plain English labels since spawn_tab_panel does not support
// localization keys yet.
```

---

## 7. Additional Findings

### 7.1 P1 -- unimplemented! / TODO with crash risk

Check `src/ui/` for `unimplemented!()` or `panic!()` calls.

### 7.2 P1 -- Bridge Module Missing

`src/ui/bridge/` does not exist (was referenced in architectural planning).
Currently there is no formal bridge between UI layer and Domain layer; the
`application/command.rs` UiCommand -> GameCommand conversion is the closest
approximation but is incomplete.

### 7.3 P2 -- UiStore Default Values

`src/ui/view_models/mod.rs:41-49`: BattleHudVm defaults use placeholder values:
- `phase_key: ""` -- empty string default means a phase_key-less battle HUD
  would display nothing rather than an informative placeholder.
- `turn_number: 0` -- intentional per comment, but should eventually use
  `Option<u32>` to distinguish "not yet loaded" from "turn 0".

### 7.4 P2 -- Settings Persistence

`src/ui/settings.rs:41`: `unwrap_or_default()` in UiSettings::load -- this
silently swallows read errors by returning default settings. A corrupt file
would be invisible to the player.

---

## 8. Summary Table

| Category | P0 | P1 | P2 | Total |
|----------|----|----|----|-------|
| Factory compliance | 1 | 0 | 4 | 5 |
| ViewModel violations | 2 | 4 | 2 | 8 |
| Hardcoded text | 1 primary + 12 screen strings | 15+ widget strings | 2 | ~30 |
| Color/font violations | 0 | 2 | 2 | 4 |
| Lifecycle/composition | 0 | 2 | 3 | 5 |
| Localization format | 0 | 1 | 1 | 2 |
| **Total** | **4** | **11** | **14** | **~29** |

---

## 9. Severity Distribution

```
P0 (must fix):  4  ████████████████████
P1 (should fix): 11 ████████████████████████████████████████████████████████
P2 (observe):   14  ██████████████████████████████████████████████████████████████████
```

---

## 10. Recommendations by Priority

### P0 -- Immediate Fixes (Architecture Violations)

1. **BattleScreen root factory**: Replace raw `commands.spawn(Node{...})` at
   `battle/mod.rs:53-63` with `spawn_panel` call, then insert `BattleScreen` and
   `Name` as extra components on the factory entity.

2. **TurnQueue domain query**: Remove `Option<Res<TurnQueue>>` from
   `battle/systems.rs:30`. EndTurn should derive `unit_id` from a ViewModel,
   not query the domain directly. Add `current_unit_id` to `BattleHudVm`.

3. **BattlePhase domain query**: Remove `Res<State<BattlePhase>>` from
   `battle/visibility.rs:20`. Zone visibility should be driven by a ViewModel
   field (e.g., `visible_zones: Vec<BattleZone>`) updated by projection.

4. **Battle turn indicator text**: Replace hardcoded `"Turn: 3    Phase: Player Turn"`
   with `spawn_localized_text` bound to `BattleHudVm` fields.

### P1 -- Should Fix (Code Quality / Technical Debt)

1. Wire `BattleHudVm` to a widget refresh system with `Dirty<BattleHudVm>`.
2. Add `Dirty<ActionMenuVm>` or equivalent to ActionMenu widget.
3. Add `Dirty<CharacterPanelVm>` refresh system for CharacterCard (exists but verify).
4. Migrate all widget hardcoded strings to localization keys and fallbacks.
5. Add `TextVariant::Display` or `theme.typography.size_display` for the title.
6. Add `theme.colors.overlay` token for modal backdrop alpha.
7. Fix localization key format: `"ui.settings.show_damage"` -> `loc::ui::*` constant.
8. Add projection wiring for zone visibility.
9. Add navigation bridge: wire `ScreenStack` push/pop, connect `UiScreenState`.

### P2 -- Observe (Track for Later)

1. Monitor that `spawn_panel` override pattern affects all screens (non-urgent).
2. Monitor empty zones Z2, Z3, Z8 for when they get widgets.
3. TabPanel localization key support.
4. UiStore default values vs Option semantics.
5. Settings file error handling.
6. Add Spawn/Despawn lifecycle hook points for overlay screens.

---

## 11. Conclusion

**Assessment: FAIL (4 P0 issues)**

The codebase has a strong architectural foundation -- Primitives factories,
Theme token system, Dirty<T> binding, and ViewModel definitions are correctly
structured. The gap is in **consistent application**:

- The BattleScreen is the most problematic area with 3 of 4 P0 violations.
- Widget-level text hardcoding is widespread (~30 locations) but the migration
  path is well-supported by existing `spawn_localized_*` factories.
- The ViewModel pipeline is partially wired (SkillPanelVm -> SkillSlot works)
  but BattleHudVm has no consumer and ActionMenu has no binding.

The 4 P0 items must be resolved before closing this review. After that,
P1 items can be addressed incrementally. Recommend re-invoking `@code-reviewer`
after P0 fixes are applied.
