//! 技能领域事件

use crate::shared::ids::{AbilityId, UnitId};
use bevy::prelude::*;

/// 技能已释放
#[derive(Message, Debug, Clone)]
pub struct SkillActivated {
    pub caster: UnitId,
    pub caster_name: String,
    pub skill_id: AbilityId,
    pub target: Option<UnitId>,
    pub target_name: Option<String>,
}
