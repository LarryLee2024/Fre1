/// 日志领域事件定义
/// 所有 INFO 级别核心业务事件必须通过这些 Message 通知 LogObserver
use bevy::prelude::*;

/// 配置加载完成事件
#[derive(Message, Debug, Clone)]
pub struct ConfigLoaded {
    pub config_type: String,
    pub id: String,
}

/// Buff 施加事件
#[derive(Message, Debug, Clone)]
pub struct BuffApplied {
    pub target: Entity,
    pub target_name: String,
    pub buff_id: String,
    pub source: Option<Entity>,
    pub remaining_turns: u32,
}

/// Buff 移除事件
#[derive(Message, Debug, Clone)]
pub struct BuffRemoved {
    pub target: Entity,
    pub target_name: String,
    pub buff_id: String,
    pub source: Option<Entity>,
}

/// Buff 过期事件
#[derive(Message, Debug, Clone)]
pub struct BuffExpired {
    pub target: Entity,
    pub target_name: String,
    pub buff_id: String,
}

/// 技能激活事件（玩家或 AI）
#[derive(Message, Debug, Clone)]
pub struct SkillActivated {
    pub caster: Entity,
    pub caster_name: String,
    pub skill_id: String,
    pub target: Entity,
    pub target_name: String,
}

/// 关卡完成事件
#[derive(Message, Debug, Clone)]
pub struct LevelCompletedEvent {
    pub level_id: String,
    pub success: bool,
}

/// 装备穿戴事件
#[derive(Message, Debug, Clone)]
pub struct EquipmentEquipped {
    pub target: Entity,
    pub target_name: String,
    pub equipment_id: String,
}

/// 装备脱卸事件
#[derive(Message, Debug, Clone)]
pub struct EquipmentUnequipped {
    pub target: Entity,
    pub target_name: String,
    pub equipment_id: String,
}

/// 物品使用事件
#[derive(Message, Debug, Clone)]
pub struct ItemUsed {
    pub user: Entity,
    pub user_name: String,
    pub item_id: String,
    pub target: Option<Entity>,
}

/// 物品转移事件
#[derive(Message, Debug, Clone)]
pub struct ItemTransferred {
    pub item_id: String,
    pub amount: u32,
    pub from_container: String,
    pub to_container: String,
}

/// 单位移动事件
#[derive(Message, Debug, Clone)]
pub struct UnitMoved {
    pub entity: Entity,
    pub unit_name: String,
    pub from: IVec2,
    pub to: IVec2,
}

/// 场景快照事件
#[derive(Message, Debug, Clone)]
pub struct SnapshotCreated {
    pub snapshot_id: String,
    pub entity_count: usize,
}
