//! Module Name: SkillPanelVm — Skill panel view model
//!
//! ViewModel for the skill panel: list of skill slots with cooldown, cost,
//! and usability state.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §3.4

use bevy::prelude::*;
use std::collections::HashMap;

/// Single skill slot in the skill panel.
///
/// Contains all data needed to render one skill button in the panel:
/// identity, cooldown state, cost, and whether it can be activated.
#[derive(Clone, Reflect, Default)]
pub struct SkillSlotVm {
    /// Skill definition ID
    pub skill_id: u32,
    /// Skill name (localization key)
    pub name_key: &'static str,
    /// Remaining cooldown turns (0 = ready)
    pub cooldown_remaining: u32,
    /// Maximum cooldown turns
    pub max_cooldown: u32,
    /// Whether the skill can be activated
    pub is_usable: bool,
    /// Action point cost
    pub ap_cost: u32,
}

/// Skill panel view model -- container for all skill slot data.
///
/// Skills are indexed by a u32 identifier (matching SkillSlotVm.skill_id)
/// for efficient lookup during widget refresh.
#[derive(Clone, Reflect, Default)]
pub struct SkillPanelVm {
    /// All skill slots, keyed by skill_id
    pub skills: HashMap<u32, SkillSlotVm>,
}
