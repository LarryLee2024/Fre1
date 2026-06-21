---
id: 09-planning.ui-layout-system
title: UI Layout Architecture Plan -- Combat HUD Zones, Widget Sizing, Action Chains
status: draft
owner: presentation-architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - ui
  - layout
  - combat-hud
  - zones
  - responsive
  - p0
  - planning
---

# UI Layout Architecture Plan

## 0. Executive Summary

The current BattleScreen spawns all widgets as children of a single Column-layout root node, causing overlap and no edge anchoring. This plan replaces that ad-hoc stacking with a **zone-based layout system**: 9 screen zones anchored to edges/corners, a game-world center area, a spacing grid system, explicit visibility rules per game phase, and a documented click-to-domain action chain for every interactive element.

**Scope**: Combat HUD only. Menu screens (MainMenu, Settings, Inventory, Shop, SaveLoad) use simpler centered layouts and are not covered here.

**Priority scale**: [P0] = blocks MVP, [P1] = needed for beta, [P2] = polish/enhancement.

---

## 1. Screen Zones (Canvas Areas)

### 1.1 Zone Map

```
 ┌──────────────────────────────────────────────────────────────┐
 │  Z1: Top-Left              Z2: Top-Center          Z3: Top-Right │
 │  TurnIndicator             PhaseText               UnitSummary  │
 │  [P1]                      [P0]                    [P2]         │
 ├──────────────────────────────────────────────────────────────┤
 │                                                               │
 │                                                               │
 │            Z4: GAME WORLD (center)                               │
 │         (6x6 grid + units + effects + cursor)                    │
 │         2D Camera Viewport                                       │
 │                                                               │
 │                                                               │
 ├──────────────────────────────────────────────────────────────┤
 │  Z5: Bottom-Left           Z6: Bottom-Center         Z7: Bottom-Right │
 │  CharacterCard             ActionMenu                SkillPanel     │
 │  [P0]                      [P0]                      [P1]          │
 │                                                               │
 │  Z8: Full-Width Bottom Bar  (optional, P2)                    │
 │  TurnOrderBar / Timeline                                        │
 └──────────────────────────────────────────────────────────────┘


 Overlay Layers (independent, NOT nested in any zone):
 ┌──────────────────────────────────────────────────────────────┐
 │  OL1: NotificationLayer (top-right stack, auto-dismiss)      │
 │  OL2: TooltipLayer (mouse-following)                          │
 │  OL3: PopupLayer (centered modal, e.g., SkillDetail)         │
 │  OL4: DebugLayer (top-left, dev-only)                        │
 └──────────────────────────────────────────────────────────────┘
```

### 1.2 Zone Definitions

| Zone ID | Name | Anchor | Offset from Edge | Size | Z-Order (within ScreenLayer) | Widgets | Visibility Rule |
|---------|------|--------|------------------|------|------------------------------|---------|-----------------|
| Z1 | Top-Left | Top-Left | top: sm, left: sm | auto-width, fixed height (56px) | 10 | TurnIndicator | Always visible in combat |
| Z2 | Top-Center | Top-Center | top: sm, horizontal center | auto-width, fixed height (56px) | 10 | PhaseText, TurnNumber | Always visible in combat |
| Z3 | Top-Right | Top-Right | top: sm, right: sm | 200px fixed width, auto height | 10 | UnitSummary, Timer | Always visible in combat |
| Z4 | Game World | Full Center | all edges: 0, padded by zone bounds | Fill remaining (total minus top row - bottom row) | 0 | Grid, Units, Effects, Cursor | Always visible in combat (rendered on separate camera) |
| Z5 | Bottom-Left | Bottom-Left | bottom: sm, left: sm | 280px x 200px fixed | 20 | CharacterCard | Visible when a unit is selected |
| Z6 | Bottom-Center | Bottom-Center | bottom: sm, horizontally centered | auto-width, auto-height (max 60px per button row) | 20 | ActionMenu | Visible during player's controllable turn |
| Z7 | Bottom-Right | Bottom-Right | bottom: sm, right: sm | 240px x auto (scrollable) | 20 | SkillPanel, EndTurnButton | SkillPanel: visible when Skill submenu is open; EndTurnButton: always during player turn |
| Z8 | Full-Width Bottom Bar | Bottom-Center | bottom: 0, left: 0, right: 0 | 100% width x 48px height | 5 | TurnOrderBar, Timeline | [P2] Always visible in combat |

**Layout assembly rule**: Zones Z1-Z3 and Z5-Z8 are positioned using `PositionType::Absolute` with `left`/`right`/`top`/`bottom` values derived from theme spacing tokens. They are NOT nested inside each other. They are siblings under the ScreenLayer root for the BattleScreen.

### 1.3 Zone Anchor Points and Screen Canvas

The screen canvas is defined as a `Canvas` resource (not a Bevy UI Node -- a logical concept):

```
Canvas dimensions: window.inner_size (logical pixels)
Canvas padding: theme.spacing.sm (8px) on all edges

Zone positions are computed from:
  - Absolute edge anchors (top, bottom, left, right)
  - Padding values from theme.spacing tokens
  - No percentage-based positioning for widgets (percentages only for the game world area)
```

### 1.4 ScreenLayer Entity Hierarchy

```
UiRoot::Screen (PositionType::Absolute, 100% x 100%)
  |
  +-- BattleScreenRoot (PositionType::Relative, 100% x 100%)
       |
       +-- Zone_TopLeft (PositionType::Absolute, top:8, left:8)
       |    +-- TurnIndicator
       |
       +-- Zone_TopCenter (PositionType::Absolute, top:8, horizontal center)
       |    +-- PhaseText
       |
       +-- Zone_TopRight (PositionType::Absolute, top:8, right:8)
       |    +-- UnitSummary [P2]
       |    +-- Timer [P2]
       |
       +-- Zone_BottomLeft (PositionType::Absolute, bottom:8, left:8)
       |    +-- CharacterCard
       |
       +-- Zone_BottomCenter (PositionType::Absolute, bottom:8, horizontal center)
       |    +-- ActionMenu
       |
       +-- Zone_BottomRight (PositionType::Absolute, bottom:8, right:8)
       |    +-- SkillPanel
       |    +-- EndTurnButton
       |
       +-- Zone_BottomBar (PositionType::Absolute, bottom:0, left:0, right:0) [P2]
            +-- TurnOrderBar
```

---

## 2. Widget Sizing and Spacing

### 2.1 Widget Sizing Reference Table

All sizes in logical pixels. Values use `theme.spacing` tokens where applicable.

| Widget | Natural Width | Natural Height | Min Width | Max Width | Internal Padding (theme tokens) | Margin from Zone Edge |
|--------|-------------|--------------|-----------|-----------|-------------------------------|----------------------|
| TurnIndicator | auto (text-driven) | 24px | 100px | none | xs (4px) horizontal | 0 |
| PhaseText | auto (text-driven) | 24px | 120px | none | sm (8px) horizontal | 0 |
| CharacterCard | 280px | 200px | 240px | 320px | lg (24px) all | sm (8px) |
| ActionMenu | auto (button-row-driven) | auto (5 x button_height + gaps) | 200px | 400px | sm (8px) | md (16px) from zone edge |
| SkillPanel | 240px | auto | 200px | 300px | md (16px) | sm (8px) |
| EndTurnButton | 140px | button_height (40px) | 120px | 180px | md (16px) horizontal, sm (8px) vertical | sm (8px) |
| TurnOrderBar [P2] | 100% of zone width | 48px | 100% | 100% | xs (4px) | 0 |

### 2.2 Widget Adjacency Behavior

When a zone contains multiple widgets, they stack according to these rules:

**Z7 (Bottom-Right)** -- contains SkillPanel + EndTurnButton:
- Stack direction: Vertical, bottom-aligned
- Order: EndTurnButton on bottom, SkillPanel above it
- Gap between them: `theme.spacing.sm` (8px)
- When SkillPanel is hidden: EndTurnButton expands to fill zone width
- When both visible: SkillPanel takes available space, EndTurnButton stays fixed at 40px height

**Z3 (Top-Right)** [P2] -- contains UnitSummary + Timer:
- Stack direction: Vertical, top-aligned
- Gap: `theme.spacing.xs` (4px)
- Both fixed-height, no expansion behavior

**Z8 (Bottom Bar)** [P2] -- contains TurnOrderBar:
- Full-width, centered content
- Horizontal scroll when turn order exceeds viewport width

### 2.3 Empty/Disabled Widget States

| Widget | Empty State | Disabled State | Behavior When Removed |
|--------|------------|----------------|----------------------|
| CharacterCard | Hidden entirely | Grey out stats (theme.colors.text_disabled) | Zone collapses to 0x0 |
| ActionMenu | All buttons disabled during enemy phase | Gray out each button individually (ButtonVariant::Disabled) | Zone collapses to 0x0 |
| SkillPanel | "No skills available" text | Gray cooldown bars at 100% | Zone collapses to 0x0 |
| EndTurnButton | Always rendered during player phase | Disabled when actions remain (optional rule) | Not removed (always present during player turn) |

---

## 3. Layout Grid System

### 3.1 Base Unit

The entire UI uses a **4px base unit**. Every spacing, padding, margin, and size value must be a multiple of 4px.

```
BASE_UNIT: 4px
Rationale: Aligns with standard display resolution scaling and the existing
UiSpacing scale which already follows this pattern (xs=4, sm=8, md=16, lg=24, xl=32, xxl=48).
```

### 3.2 Spacing Scale

| Token | Value (px) | Base Unit Multiples | Usage |
|-------|-----------|---------------------|-------|
| `theme.spacing.xs` | 4 | 1x | Internal button padding, tight icon spacing, gap between closely related elements |
| `theme.spacing.sm` | 8 | 2x | Zone margins, panel content padding, gap between widget groups |
| `theme.spacing.md` | 16 | 4x | Standard panel padding, modal content padding, section spacing |
| `theme.spacing.lg` | 24 | 6x | Card internal padding, large section breaks |
| `theme.spacing.xl` | 32 | 8x | Screen-level padding, menu spacing |
| `theme.spacing.xxl` | 48 | 12x | Large screen margins, title spacing |

### 3.3 Size Tokens (Sizing Constants)

These are **not** in the current `UiSpacing` struct. This plan defines them as a separate `UiSizing` resource to be added:

```rust
/// New resource: layout sizing constants
pub struct UiSizing {
    /// Zone minimum dimensions
    pub zone_min_width_top: f32,      // 100.0
    pub zone_min_height_top: f32,     // 56.0
    pub zone_min_width_bottom_left: f32,  // 240.0
    pub zone_min_height_bottom_left: f32, // 180.0
    
    /// Widget sizing
    pub character_card_width: f32,    // 280.0
    pub skill_panel_width: f32,       // 240.0
    pub end_turn_button_width: f32,   // 140.0
    pub turn_order_bar_height: f32,   // 48.0
    
    /// Canvas edge padding (may differ from theme.spacing)
    pub canvas_padding: f32,          // 8.0
}
```

The `UiSizing` resource is theme-agnostic (same values for dark/light). It is registered in the UI plugin chain alongside `Theme`.

### 3.4 Primitive Sizing Derivation

All primitives derive their sizing from the spacing scale:

| Primitive | Width Derivation | Height Derivation | Padding |
|-----------|-----------------|-------------------|---------|
| **Button (Primary/Secondary)** | auto (content-driven) or fixed via props | `theme.spacing.button_height` (40px) | horizontal: `md` (16px), vertical: `xs` (4px) |
| **Button (Ghost)** | auto | `theme.spacing.button_height` (40px) | horizontal: `sm` (8px), vertical: `xs` (4px) |
| **Button (Danger)** | same as Primary | `theme.spacing.button_height` (40px) | same as Primary |
| **Panel (Basic)** | 100% parent or fixed | auto | all: `md` (16px) |
| **Panel (Card)** | fixed or auto | auto | all: `lg` (24px) |
| **Panel (Group)** | 100% parent | auto | all: `md` (16px) |
| **Text (Body)** | auto | line-height = font_size * 1.4 | 0 |
| **Text (Caption)** | auto | line-height = font_size * 1.3 | 0 |
| **ProgressBar** | 100% parent | 16px (fixed) | 0 |
| **List (Vertical)** | 100% parent or auto | auto | 0 (children provide spacing via `gap`) |
| **Toggle** | 44px (min_touch_target) | 24px | 0 |
| **TextInput** | 200px or fill parent | `theme.spacing.button_height` (40px) | horizontal: `sm` (8px) |

### 3.5 Gap Constants

Widget lists use `column_gap` / `row_gap` on the container Node, not margin on children:

| Container Type | Gap Token | Value |
|---------------|-----------|-------|
| Button group (horizontal) | row_gap | `theme.spacing.sm` (8px) |
| List (vertical, e.g., ActionMenu) | column_gap | `theme.spacing.xs` (4px) |
| Form group (vertical) | column_gap | `theme.spacing.md` (16px) |
| Tab bar (horizontal) | row_gap | 0 (tabs touch) |

---

## 4. Click to Action Chain

### 4.1 Action Chain Architecture

Every interactive UI element follows the same unidirectional pipeline:

```
User Click
  |
  v
[Bevy Interaction::Pressed]
  |
  v
[Primitive Layer]
  button_interaction_system (observer or event)
  -> ButtonClicked { entity }
  |
  v
[Screen Layer Observer]
  on_<screen>_button_clicked (On<ButtonClicked>)
  -> match ActionComponent -> UiCommand
  -> commands.trigger(UiCommand)
  |
  v
[Bridge Layer]
  process_ui_commands (On<UiCommand>)
  -> cmd.into_game_command() -> Option<GameCommand>
  -> CommandQueue.push(game_cmd)
  |
  v
[Domain Layer]
  command_processing_system
  -> default_command_handler
  -> domain handler -> domain event
```

### 4.2 Action Chains by Widget

#### 4.2.1 EndTurnButton (BattleScreen, Zone Z7)

```
EndTurnButton click
  -> ButtonClicked { entity: E }
  -> on_battle_button_clicked observer
     -> Query::get(E) for BattleAction
     -> BattleAction::EndTurn
     -> UiCommand::EndTurn
     -> commands.trigger(UiCommand::EndTurn)
     |
     -> process_ui_commands (On<UiCommand::EndTurn>)
        -> cmd.into_game_command() = Some(GameCommand::EndTurn { unit_id: "" })
        -> CommandQueue.push(GameCommand::EndTurn { unit_id: "" })
        |
        -> command_processing_system
           -> processor.handle(GameCommand::EndTurn)
           -> domain: TurnManager.end_turn()
           -> domain event: TurnPhaseChanged { new_phase: EnemyPhase }
           |
           -> [Projection] BattleProjection.on_turn_phase_changed(event)
              -> update BattleHudVm.phase_key
              -> set Dirty<BattleHudVm>
              |
              -> [ViewModel refresh system]
                 -> TurnIndicator widget reads BattleHudVm
                 -> ActionMenu hides (visibility rules)
                 -> CharacterCard hides if no unit selected
```

#### 4.2.2 ActionMenu.Attack (BattleScreen, Zone Z6)

```
AttackButton click
  -> ButtonClicked { entity: E }
  -> on_battle_button_clicked observer
     -> Query::get(E) for ActionType
     -> ActionType::Attack
     -> UiCommand::SelectTarget(0)  // target_id populated later
     -> commands.trigger(UiCommand::SelectTarget(0))
     |
     -> process_ui_commands (On<UiCommand::SelectTarget>)
        -> cmd.into_game_command() = None (needs context)
        -> [UI handles internally: enter target-selection mode]
        |
     -> BattleScreen enters "Targeting" sub-state
        -> Grid cells become clickable
        -> Cursor changes to crosshair
        -> ActionMenu dims
        -> CharacterCard shows predicted damage
```

#### 4.2.3 ActionMenu.Skill (BattleScreen, Zone Z6)

```
SkillButton click
  -> ButtonClicked { entity: E }
  -> ActionType::Skill
  -> [UI toggles SkillPanel visibility]
  -> SkillPanel appears in Z7
  -> ActionMenu remains visible
  
  --- then, if a specific skill is clicked ---
  
  SkillSlot.UseButton click
  -> ButtonClicked { entity: E2 }
  -> SkillSlotAction::Use
  -> UiCommand::CastSkill { skill_def_id, target_id: "", caster_id }
  -> commands.trigger(UiCommand::CastSkill { ... })
  |
  -> process_ui_commands
     -> cmd.into_game_command() = Some(GameCommand::CastSpell { ... })
     -> CommandQueue.push(GameCommand::CastSpell { ... })
     |
     -> domain: SpellSystem.cast()
     -> domain event: SpellCast { ... }
     -> Projection -> BattleHudVm update (MP cost)
     -> Dirty<BattleHudVm> -> CharacterCard refreshes MP bar
```

#### 4.2.4 ActionMenu.Item (BattleScreen, Zone Z6)

```
ItemButton click
  -> ButtonClicked { entity: E }
  -> ActionType::Item
  -> UiCommand::OpenScreen(ScreenType::Inventory)
  -> commands.trigger(UiCommand::OpenScreen(ScreenType::Inventory))
  |
  -> [Navigation system]
     -> ScreenStack.push(ScreenType::Inventory)
     -> InventoryScreen spawns
     |
  -> [From InventoryScreen, when item used in combat]
     -> InventoryItemRow.UseButton click
     -> InventoryGridAction::UseItem(item_instance_id)
     -> UiCommand::UseItem { item_instance_id, user_id, target_id }
     -> CommandQueue.push(GameCommand::UseItem { ... })
     -> domain event -> Projection -> ViewModel update
```

#### 4.2.5 ActionMenu.Wait (BattleScreen, Zone Z6)

```
WaitButton click
  -> ButtonClicked { entity: E }
  -> ActionType::Wait
  -> UiCommand::EndTurn  // same as EndTurn in MVP
  -> [same chain as EndTurnButton]
```

#### 4.2.6 MainMenu.NewGame (MainMenuScreen)

```
NewGameButton click
  -> ButtonClicked { entity: E }
  -> on_main_menu_button_handler observer
     -> MenuAction::NewGame
     -> UiCommand::NewGame
     -> commands.trigger(UiCommand::NewGame)
     |
     -> process_ui_commands
        -> cmd.into_game_command() = Some(GameCommand::NewGame)
        -> CommandQueue.push(GameCommand::NewGame)
        |
        -> [App State Transition]
           -> GameState::Combat
           -> BattleScreen spawns
```

#### 4.2.7 MainMenu.Load (MainMenuScreen)

```
LoadGameButton click
  -> ButtonClicked { entity: E }
  -> MenuAction::LoadGame
  -> UiCommand::OpenScreen(ScreenType::SaveLoad)
  -> [Navigation]
     -> ScreenStack.push(ScreenType::SaveLoad)
     -> SaveLoadScreen spawns
```

#### 4.2.8 MainMenu.Settings (MainMenuScreen)

```
SettingsButton click
  -> ButtonClicked { entity: E }
  -> MenuAction::Settings
  -> UiCommand::OpenScreen(ScreenType::Settings)
  -> [Navigation]
     -> ScreenStack.push(ScreenType::Settings)
     -> SettingsScreen spawns
```

#### 4.2.9 Grid Cell Click (BattleScreen, Zone Z4) [P1]

```
GridCell click  (handled by separate mouse input, not UI Button)
  -> WorldClick { world_position }
  -> [Input system]
     -> GridPos from world_position
     -> UiCommand::MoveToPosition { unit_id, x, y }
     -> OR UiCommand::SelectTarget(character_id) if in targeting mode
  |
  -> [Domain]
     -> CommandQueue.push(GameCommand::MoveUnit { ... })
     -> OR domain resolves target selection
```

### 4.3 Action Chain Summary Table

| Widget | Click Output | Action Component | UiCommand | GameCommand | Domain Target |
|--------|------------|-----------------|-----------|-------------|---------------|
| EndTurnButton | ButtonClicked | BattleAction::EndTurn | EndTurn | EndTurn | TurnManager |
| AttackButton | ButtonClicked | ActionType::Attack | SelectTarget(0) | None (UI internal) | -- |
| DefendButton | ButtonClicked | ActionType::Defend | EndTurn (MVP) | EndTurn | TurnManager |
| SkillButton | ButtonClicked | ActionType::Skill | OpenScreen / toggle submenu | None (UI internal) | -- |
| SkillSlot.UseButton | ButtonClicked | SkillSlotAction::Use | CastSkill{...} | CastSpell | SpellSystem |
| ItemButton | ButtonClicked | ActionType::Item | OpenScreen(Inventory) | None (navig.) | -- |
| WaitButton | ButtonClicked | ActionType::Wait | EndTurn | EndTurn | TurnManager |
| NewGameButton | ButtonClicked | MenuAction::NewGame | NewGame | NewGame | App init |
| LoadGameButton | ButtonClicked | MenuAction::LoadGame | OpenScreen(SaveLoad) | None (navig.) | -- |
| SettingsButton | ButtonClicked | MenuAction::Settings | OpenScreen(Settings) | None (navig.) | -- |

---

## 5. Visibility Rules

### 5.1 Game Phase Visibility Matrix

Visibility is driven by the `BattlePhase` domain state, projected into `BattleHudVm.phase_key` and consumed by UI systems.

| Widget | PlayerPhase | EnemyPhase | TransitionPhase (between turns) | Victory/Defeat |
|--------|------------|-----------|-------------------------------|----------------|
| TurnIndicator (Z1) | Visible | Visible | Visible | Hidden [P2] |
| PhaseText (Z2) | Visible ("Player Turn") | Visible ("Enemy Turn") | Visible ("Processing...") | Visible ("Victory!"/"Defeat") |
| UnitSummary (Z3) [P2] | Visible | Visible | Visible | Hidden |
| Grid + Units (Z4) | Visible | Visible | Visible | Visible |
| CharacterCard (Z5) | Visible when unit selected | Visible when hovered [P2] | Same as PlayerPhase | Hidden |
| ActionMenu (Z6) | **Visible** (all buttons enabled) | **Hidden** | Hidden | Hidden |
| SkillPanel (Z7) | Visible when Skill submenu open | Hidden | Hidden | Hidden |
| EndTurnButton (Z7) | Visible | Hidden | Hidden | Hidden |
| TurnOrderBar (Z8) [P2] | Visible | Visible | Visible | Hidden |

### 5.2 Unit Selection Visibility Rules

| Condition | CharacterCard | ActionMenu | SkillPanel | Grid Highlight |
|-----------|--------------|------------|------------|----------------|
| No unit selected | Hidden | Visible (last selected unit) | Hidden | None |
| Own unit selected | Visible (selected unit stats) | Visible (all actions) | Hidden until Skill clicked | Blue highlight on selected + movement range |
| Enemy unit selected | Visible (enemy stats) | Visible (attack action only) [P1] | Hidden | Red highlight on target |
| Ally unit selected | Visible (ally stats) | Visible (support actions) [P2] | Hidden | Green highlight |

### 5.3 Sub-state Visibility Rules

| Sub-state | Affected Widgets | Visibility Change |
|-----------|-----------------|-------------------|
| Targeting mode (after Attack/Skill) | ActionMenu | Dimmed (Interaction::None on buttons except Cancel) |
| Targeting mode | Grid cells | Highlighted valid targets |
| Targeting mode | CharacterCard | Always visible (shows selected unit) |
| Targeting mode | SkillPanel | Visible if Skill was selected (shows which skill) |
| Skill submenu open | ActionMenu | Remains visible |
| Skill submenu open | SkillPanel | **Visible** (in Z7) |
| Skill submenu open | EndTurnButton | Remains visible |
| Modal open (any) | All zones in ScreenLayer | Behind modal backdrop (interaction blocked, visually dimmed) |

### 5.4 Visibility Implementation Pattern

Visibility is NOT implemented by spawning/despawning widgets. Widgets are persistent while the BattleScreen is alive. Visibility is controlled via `Visibility` component:

```rust
// System: update zone visibility based on BattlePhase (pseudocode)
fn update_zone_visibility(
    battle_phase: Res<State<BattlePhase>>,
    hud_vm: Res<BattleHudVm>,
    mut query: Query<(&mut Visibility, &ZoneId)>,
) {
    for (mut vis, zone) in query.iter_mut() {
        *vis = match zone {
            ZoneId::ActionMenu if battle_phase.get() != BattlePhase::PlayerTurn => Visibility::Hidden,
            ZoneId::EndTurnBtn if battle_phase.get() != BattlePhase::PlayerTurn => Visibility::Hidden,
            ZoneId::CharacterCard if hud_vm.selected_unit.is_none() => Visibility::Hidden,
            _ => Visibility::Inherited,
        };
    }
}
```

This pattern avoids ECS spawn/despawn churn every phase transition, consistent with the performance principle of persistent widgets with dirty flags.

---

## 6. Mobile and Responsive Considerations

### 6.1 Minimum Zone Sizes

For responsive scaling, each zone has a minimum size. If the viewport shrinks below the sum of minimums, widgets begin to overlap (intentional -- not all screens aspect ratios are supported for MVP).

| Zone | Min Width | Min Height | Behaviour Below Minimum |
|------|-----------|-----------|------------------------|
| Z1 (Top-Left) | 100px | 24px | Text truncation |
| Z2 (Top-Center) | 120px | 24px | Text truncation |
| Z3 (Top-Right) | 160px | 24px | Collapse/hide |
| Z4 (Game World) | 320px | 240px | Viewport letterboxing |
| Z5 (Bottom-Left) | 240px | 180px | Collapse/hide content |
| Z6 (Bottom-Center) | 200px | auto | Buttons wrap to 2 columns |
| Z7 (Bottom-Right) | 200px | auto | Scroll within panel |
| Z8 (Bottom Bar) | 100% | 32px (shrunk) | Text truncation |

### 6.2 Breakpoints

| Breakpoint | Window Width | Layout Adjustments |
|------------|-------------|-------------------|
| Desktop (standard) | >= 1280px | Full 9-zone layout as designed |
| Desktop (small) | 1024-1279px | Z3 (Top-Right) hides; Z8 shrinks to 32px height |
| Tablet landscape | 768-1023px | Z3 hides; Z5 (CharacterCard) reduces to 200px width; Z8 collapses to icon-only |
| Tablet portrait | 480-767px | Z1+Z2 merge; Z5/Z6/Z7 reflow to single row at bottom; buttons shrink to icon-only |
| Phone | < 480px | [P2] Not officially supported for MVP; viewport letterboxed to minimum 480px |

### 6.3 Reflow Rules

When screen width decreases:

1. **Z3 (Top-Right)** is the first to hide (at < 1024px). Its content (Timer, UnitSummary) relocates to Z2 or into a collapsible dropdown accessed via Z1.

2. **Z5/Z6/Z7 reflow** at tablet portrait:
   - Bottom row becomes a horizontal scrollable strip
   - CharacterCard reduces to name + HP bar only (compact variant)
   - ActionMenu buttons become icon-only with tooltips
   - SkillPanel collapses to a grid icon that opens a full-overlay skill browser

3. **Font scaling**:
   - Body text: clamp(14px, 2vw, 18px)
   - Caption text: clamp(12px, 1.5vw, 14px)
   - Button text: clamp(12px, 1.5vw, 16px)
   - Title text: clamp(24px, 4vw, 48px)
   - Font scaling uses `clamp()` style min/max calculations applied via a responsive system that reads window width

### 6.4 Touch Target Minimum

All interactive elements must have a minimum touch target of `theme.spacing.min_touch_target` (44px) in both dimensions. This is enforced at the primitive level (Button factory, Toggle factory, etc.):

```
If a Button's rendered size < 44px in either dimension:
  -> Increase padding or min_size to meet 44px threshold
  -> Warning logged (only in dev builds)
```

---

## 7. Implementation Phasing

### 7.1 Phase 1 (MVP -- Current Sprint) [P0]

Items marked [P0] in the tables above.

1. Create `UiSizing` resource with zone dimension constants.
2. Refactor `BattleScreen` to use absolute-positioned zones instead of single Column layout.
   - Zone containers (Z1, Z2, Z5, Z6, Z7) as `PositionType::Absolute` children of `BattleScreenRoot`.
   - Game world area (Z4) is the existing 2D camera viewport -- no change needed.
3. Port existing widgets into zones:
   - TurnIndicator text -> Z2 (Top-Center)
   - CharacterCard -> Z5 (Bottom-Left)
   - ActionMenu -> Z6 (Bottom-Center)
   - EndTurnButton -> Z7 (Bottom-Right)
4. Implement Visibility system for BattlePhase transitions.
5. Wire EndTurnButton action chain through UiCommand (already partially done in `systems.rs`).

### 7.2 Phase 2 (Near Term) [P1]

1. SkillPanel wiring: toggle SkillPanel visibility on ActionMenu.Skill click.
2. ActionMenu.Attack enters targeting mode (UI sub-state).
3. Grid cell click handling for movement and targeting.
4. TurnIndicator widget (Z1) showing turn queue count and current unit.
5. Bottom bar (Z8) for TurnOrderBar.

### 7.3 Phase 3 (Post-MVP) [P2]

1. Responsive reflow at breakpoints.
2. CharacterCard compact variant for small screens.
3. UnitSummary zone (Z3).
4. Animations: zone entrance/exit, button press, phase transition.
5. TurnOrderBar with horizontal scrolling.

---

## 8. Validation Rules

| # | Rule | Check |
|---|------|-------|
| LAY-VAL-01 | Every zone has PositionType::Absolute | Zone containers must NOT use PositionType::Relative or default |
| LAY-VAL-02 | No widget is a direct child of BattleScreen root | All widgets must be inside a zone container |
| LAY-VAL-03 | Visibility uses Visibility component, not spawn/despawn | No `despawn_recursive()` on phase change |
| LAY-VAL-04 | Zone positions use theme.spacing tokens | No hardcoded `Val::Px(8)` for zone margins |
| LAY-VAL-05 | Every interactive element has an action chain documented | See Section 4 of this document |
| LAY-VAL-06 | Touch targets >= 44px | Button/Toggle/SelectList factories enforce min_touch_target |
| LAY-VAL-07 | Sizing is a multiple of BASE_UNIT (4px) | All `Val::Px(n)` values have n % 4 == 0 |

---

## 9. Open Questions and Risks

| Question | Impact | Proposed Resolution |
|----------|--------|-------------------|
| How does the grid world area (Z4) communicate with the UI for targeting? | High -- targeting mode crosses camera boundary | Input system captures world-space clicks, translates to grid coords, emits UiCommand via observer |
| Should ActionMenu hide or dim during targeting? | Medium | **Decision**: Dim, not hide. Buttons become non-interactive except Cancel. This keeps layout stable. |
| What happens when CharacterCard is hidden and then shown again? | Low | Widget is persistent with `Visibility::Hidden`. No re-spawn needed. Dirty flag triggers ViewModel refresh. |
| How does SkillPanel know which unit's skills to show? | Medium | CharacterCard.SelectedUnitId -> SkillPanelVm.selected_unit_id. Projection filters skills by unit. |
| Font size scaling strategy for localization (longer text in German vs shorter in Chinese)? | Medium | Use `min-width` on text containers with text-overflow handling. Avoid fixed-width text containers for localizable content. |

---

## Appendix A: Zone Positioning Reference (Pseudo-code)

```rust
// Zone positioning constants derived from theme and UiSizing
fn configure_zones(theme: &Theme, sizing: &UiSizing) -> ZonePositions {
    ZonePositions {
        top_left: Node {
            position_type: PositionType::Absolute,
            top: Val::Px(theme.spacing.sm),
            left: Val::Px(theme.spacing.sm),
            width: Val::Auto,
            height: Val::Px(sizing.zone_min_height_top),
            ..default()
        },
        top_center: Node {
            position_type: PositionType::Absolute,
            top: Val::Px(theme.spacing.sm),
            left: Val::Percent(50.0),
            // transform: translateX(-50%) via Node::transform
            width: Val::Auto,
            height: Val::Px(sizing.zone_min_height_top),
            ..default()
        },
        bottom_left: Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(theme.spacing.sm),
            left: Val::Px(theme.spacing.sm),
            width: Val::Px(sizing.character_card_width),
            height: Val::Auto,
            ..default()
        },
        bottom_center: Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(theme.spacing.sm),
            left: Val::Percent(50.0),
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        bottom_right: Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(theme.spacing.sm),
            right: Val::Px(theme.spacing.sm),
            width: Val::Px(sizing.skill_panel_width),
            height: Val::Auto,
            ..default()
        },
    }
}
```

## Appendix B: Existing Code Dependencies

This plan depends on and extends the following existing code:

| File | Dependency |
|------|-----------|
| `src/ui/overlay/layers.rs` | Provides ScreenLayer root entity -- zone containers are children of this |
| `src/ui/theme/spacing.rs` | UiSpacing scale used for all zone offsets and widget padding |
| `src/ui/theme/resource.rs` | Theme resource consumed by all factory functions |
| `src/ui/screens/battle/mod.rs` | spawn_battle_screen -- must be refactored to build zones |
| `src/ui/screens/battle/systems.rs` | BattleAction enum and observer -- extended for all button types |
| `src/ui/application/command.rs` | UiCommand enum -- existing variants cover all actions |
| `src/ui/application/bridge.rs` | process_ui_commands -- no changes needed |
| `src/ui/view_models/battle_hud.rs` | BattleHudVm -- extended with selected_unit field |
| `src/ui/widgets/action_menu/factory.rs` | spawn_action_menu -- no changes, just repositioned |
| `src/ui/widgets/character_card/factory.rs` | spawn_character_card -- no changes, just repositioned |
| `src/ui/widgets/skill_slot/factory.rs` | spawn_skill_slot -- no changes, just repositioned |

No existing code is deleted. Only `spawn_battle_screen` is restructured to build zones.
