---
name: battle-hud-data-flow
description: BattleHudData Resource + UiBinding + Dirty consumption system for BattleScreen character card data flow
metadata:
  type: project
---

# BattleHudData Flow (2026-06-22)

Established the first step of UiStore-to-Widget data flow for BattleScreen's CharacterCard. Previously, `spawn_character_card` was called with hardcoded values ("Aria", 5, 80.0, 100.0, 40.0, 50.0).

## Architecture

Three layers activated:

1. **BattleHudData Resource** (`src/ui/view_models/battle_hud.rs`) -- Temporary data bridge between ViewModel and widgets. Registered as Resource + Reflect in `ScreenPlugin`. Populated with sensible defaults matching previously hardcoded values.

2. **UiBinding Components** (`src/ui/screens/battle/mod.rs`) -- `UiBinding::Hp` and `UiBinding::Mp` inserted on CharacterCard container entity, enabling future Dirty-based refresh pipeline.

3. **Dirty Consumption System** (`src/ui/screens/battle/systems.rs::on_dirty_battle_hud`) -- Skeleton system registered in Update (gated by `GameState::Combat`). Currently consumes Dirty<BattleHudVm> markers but does no sync -- marked TODO[P2][UI][2026-07-21] for when Projection system is fully wired.

## Files Changed
- `src/ui/view_models/battle_hud.rs` -- Added BattleHudData struct
- `src/ui/screens/battle/mod.rs` -- Added UiBinding import + BattleHudData param + UiBinding::Hp/Mp on char_card
- `src/ui/screens/battle/systems.rs` -- Added on_dirty_battle_hud system with imports
- `src/ui/screens/mod.rs` -- ScreenPlugin: init_resource + register_type + system registration

## Why this approach
The user explicitly requested a "temporary transition solution" since the Projection system is not fully ready. BattleHudData bridges the gap -- data comes from a Resource (defined in view_models module) rather than hardcoded, and the Dirty<T> infrastructure is activated but dormant.

## How to verify
The CharacterCard still shows "Aria/Lv.5/80HP/40MP" at runtime because BattleHudData.Default has those values. The difference: the values are now read from `Res<BattleHudData>` rather than literal constants.
