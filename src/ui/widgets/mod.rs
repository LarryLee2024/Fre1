//! Module Name: Widgets — Gameplay UI controls
//!
//! Composes primitives-layer components into game-concept controls.
//! This layer is the sole consumer of Primitives; no direct Node/Button
//! manipulation allowed.
//!
//! Current widgets:
//! - SkillSlot — Skill card control
//! - ActionMenu — Battle action menu (Attack, Defend, Skill, Item, Wait)
//! - CharacterCard — Character stats card (name, level, HP/MP bars)
//!
//! Future widgets:
//! - BuffIcon
//! - InventoryGrid
//! - BattleLog
//!
//! See `docs/06-ui/02-design-system/widget-composites.md`

pub mod action_menu;
pub mod character_card;
pub mod skill_slot;

use bevy::prelude::*;

use self::action_menu::ActionMenuPlugin;
use self::character_card::CharacterCardPlugin;
use self::skill_slot::SkillSlotPlugin;

/// WidgetsPlugin — registers all gameplay UI controls
///
/// Added after PrimitivesPlugin, before ScreenPlugin.
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SkillSlotPlugin, ActionMenuPlugin, CharacterCardPlugin));
    }
}
