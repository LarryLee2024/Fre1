//! 装备领域事件

use crate::shared::ids::UnitId;
use bevy::prelude::*;

/// 装备已穿戴
#[derive(Message, Debug, Clone)]
pub struct EquipmentEquipped {
    pub unit_id: UnitId,
    pub unit_name: String,
    pub equipment_id: String,
}

/// 装备已卸下
#[derive(Message, Debug, Clone)]
pub struct EquipmentUnequipped {
    pub unit_id: UnitId,
    pub unit_name: String,
    pub equipment_id: String,
}
