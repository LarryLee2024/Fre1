//! Module Name: CharacterPanelVm — Character panel view model
//!
//! ViewModel for the character detail panel: stats, level, name.
//! Used when inspecting a character's full information.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §3.4

use bevy::prelude::*;

/// Character panel view model -- the sole data source for character panel
/// widgets.
///
/// Fields use simple primitives and never reference domain types.
/// Text fields use `&'static str` as a text key for subsequent localization
/// lookup.
#[derive(Clone, Reflect, Default)]
pub struct CharacterPanelVm {
    /// Character entity ID (0 = none selected)
    pub character_id: u32,
    /// Character name (localization key)
    pub name_key: &'static str,
    /// Character level
    pub level: u32,
    /// Current HP
    pub hp: f32,
    /// Maximum HP
    pub max_hp: f32,
    /// Current MP
    pub mp: f32,
    /// Maximum MP
    pub max_mp: f32,
}
