//! Module Name: BattleHudVm — Battle HUD view model
//!
//! ViewModel for the in-combat HUD: HP/MP bars, turn counter, phase indicator.
//! This is the exclusive data source for battle HUD widgets.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §3.4

use bevy::prelude::*;

/// Battle HUD view model -- the sole data source for battle HUD widgets.
///
/// Fields use simple primitives (f32, u32, &'static str) and never reference
/// domain types.  Text fields use `&'static str` as a text key for subsequent
/// localization lookup.
#[derive(Clone, Reflect, Default)]
pub struct BattleHudVm {
    /// Current HP
    pub hp: f32,
    /// Maximum HP
    pub max_hp: f32,
    /// Current MP
    pub mp: f32,
    /// Maximum MP
    pub max_mp: f32,
    /// Current turn number
    pub turn_number: u32,
    /// Phase description (localization key)
    pub phase_key: &'static str,
}
