//! 战斗领域事件

use crate::shared::ids::{AbilityId, UnitId};
use bevy::prelude::*;

/// 伤害已结算
#[derive(Message, Debug, Clone)]
pub struct DamageDealt {
    pub attacker: UnitId,
    pub attacker_name: String,
    pub target: UnitId,
    pub target_name: String,
    pub amount: i32,
    pub is_skill: bool,
    pub skill_id: Option<AbilityId>,
    pub is_critical: bool,
}

/// 单位已死亡
#[derive(Message, Debug, Clone)]
pub struct CharacterDied {
    pub unit_id: UnitId,
    pub name: String,
    pub killed_by: Option<UnitId>,
    pub faction: String,
}

/// 治疗已结算
#[derive(Message, Debug, Clone)]
pub struct HealApplied {
    pub target: UnitId,
    pub target_name: String,
    pub amount: i32,
    pub source: Option<UnitId>,
}

/// 晕眩已施加
#[derive(Message, Debug, Clone)]
pub struct StunApplied {
    pub target: UnitId,
    pub target_name: String,
    pub duration: u32,
}

/// DoT 伤害已结算
#[derive(Message, Debug, Clone)]
pub struct DotApplied {
    pub target: UnitId,
    pub target_name: String,
    pub amount: i32,
}

/// HoT 治疗已结算
#[derive(Message, Debug, Clone)]
pub struct HotApplied {
    pub target: UnitId,
    pub target_name: String,
    pub amount: i32,
}
