// 战斗事件 Message：逻辑层通知，表现层响应
// 遵循「Logic 发消息，Presentation 响应」原则

use crate::character::{Faction, GridPosition, Unit, UnitName};
use crate::turn::TurnOrder;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 角色死亡消息
#[derive(Message, Debug, Clone)]
pub struct CharacterDied {
    pub entity: Entity,
    /// 死亡单位名称
    pub name: String,
    /// 死亡单位阵营
    pub faction: Faction,
}

/// 伤害应用消息：逻辑层发送，VFX/日志/表现层响应
#[derive(Message, Debug, Clone)]
pub struct DamageApplied {
    pub target: Entity,
    /// 伤害目标名称
    pub target_name: String,
    /// 目标阵营
    pub target_faction: Faction,
    /// 攻击者名称
    pub attacker_name: String,
    /// 攻击者阵营
    pub attacker_faction: Faction,
    /// 伤害量
    pub amount: i32,
    /// 是否技能攻击
    pub is_skill: bool,
    /// 地形标签
    pub terrain_label: String,
    /// 目标格子坐标
    pub target_coord: IVec2,
}

/// 治疗应用消息
#[derive(Message, Debug, Clone)]
pub struct HealApplied {
    pub target: Entity,
    /// 目标名称
    pub target_name: String,
    /// 治疗量
    pub amount: i32,
}

/// 晕眩消息
#[derive(Message, Debug, Clone)]
pub struct StunApplied {
    pub target: Entity,
    pub target_name: String,
}

/// DoT 伤害消息
#[derive(Message, Debug, Clone)]
pub struct DotApplied {
    pub target: Entity,
    pub target_name: String,
    pub amount: i32,
    pub target_coord: IVec2,
}

/// HoT 治疗消息
#[derive(Message, Debug, Clone)]
pub struct HotApplied {
    pub target: Entity,
    pub target_name: String,
    pub amount: i32,
}

/// 响应 CharacterDied 消息：从行动队列移除死亡单位，despawn 实体
pub fn on_character_died(
    mut commands: Commands,
    mut died_reader: MessageReader<CharacterDied>,
    mut turn_order: ResMut<TurnOrder>,
) {
    for msg in died_reader.read() {
        bevy::log::info!("[Message] CharacterDied: {} ({:?})", msg.name, msg.faction);
        turn_order.queue.retain(|&e| e != msg.entity);
        commands.entity(msg.entity).try_despawn();
    }
}
