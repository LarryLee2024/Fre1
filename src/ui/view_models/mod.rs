//! Module Name: ViewModels — UI data projection targets
//!
//! Defines all ViewModel types (Vm suffix) and the UiStore unified container.
//! ViewModels are the exclusive data source for all widgets -- widgets never
//! query domain components directly.
//!
//! UiStore is a global Resource that holds every ViewModel as a flat field.
//! Projection functions write to UiStore; Widget systems read from UiStore.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §3, §6

pub mod battle_hud;
pub mod character_panel;
pub mod skill_panel;

use bevy::prelude::*;

use self::battle_hud::BattleHudVm;
use self::character_panel::CharacterPanelVm;
use self::skill_panel::SkillPanelVm;

/// UiStore -- unified ViewModel container, the sole data source for widgets.
///
/// # Architecture principles
/// - UiStore is the only data source widgets may read
/// - Widgets are forbidden from querying domain components directly
/// - Projection pure functions write to UiStore and mark dirty flags
/// - Widget systems detect dirty flags and refresh only when needed
///
/// # Registration
/// UiStore must be registered via `app.register_type::<UiStore>()` and
/// `app.init_resource::<UiStore>()` in the plugin.
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiStore {
    /// Battle HUD state (HP/MP bars, turn counter, phase)
    pub battle_hud: BattleHudVm,
    /// Character detail panel state
    pub character_panel: CharacterPanelVm,
    /// Skill panel state (skill slots, cooldowns, costs)
    pub skill_panel: SkillPanelVm,
}

impl Default for UiStore {
    fn default() -> Self {
        Self {
            battle_hud: BattleHudVm::default(),
            character_panel: CharacterPanelVm::default(),
            skill_panel: SkillPanelVm::default(),
        }
    }
}
