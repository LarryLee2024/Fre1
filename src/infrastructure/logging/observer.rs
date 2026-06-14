/// LogObserver：监听领域事件，输出 TracingLog
/// 遵循「日志是领域事件的消费者」原则（宪法 §14.8.1）
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::core::battle::{
    CharacterDied, DamageApplied, DotApplied, HealApplied, HotApplied, StunApplied,
};
use crate::core::turn::{TurnEnded, TurnStarted};

use super::events::{
    BuffApplied, BuffExpired, BuffRemoved, ConfigLoaded, EquipmentEquipped, EquipmentUnequipped,
    ItemTransferred, ItemUsed, LevelCompletedEvent, SkillActivated, SnapshotCreated, UnitMoved,
};

// ==================== 战斗事件日志 ====================

/// 监听 DamageApplied，输出伤害日志
pub fn log_damage_applied(mut reader: MessageReader<DamageApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "battle",
            event = "damage_applied",
            attacker = ?msg.attacker,
            attacker_name = %msg.attacker_name,
            target = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            is_skill = msg.is_skill,
            "伤害应用"
        );
    }
}

/// 监听 HealApplied，输出治疗日志
pub fn log_heal_applied(mut reader: MessageReader<HealApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "battle",
            event = "heal_applied",
            target = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "治疗应用"
        );
    }
}

/// 监听 CharacterDied，输出死亡日志
pub fn log_character_died(mut reader: MessageReader<CharacterDied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "battle",
            event = "unit_died",
            entity = ?msg.entity,
            name = %msg.name,
            faction = ?msg.faction,
            "角色已死亡"
        );
    }
}

/// 监听 StunApplied，输出晕眩日志
pub fn log_stun_applied(mut reader: MessageReader<StunApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "stun_applied",
            target = ?msg.target,
            target_name = %msg.target_name,
            "晕眩施加"
        );
    }
}

/// 监听 DotApplied，输出 DoT 日志
pub fn log_dot_applied(mut reader: MessageReader<DotApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "dot_applied",
            target = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "DoT 伤害结算"
        );
    }
}

/// 监听 HotApplied，输出 HoT 日志
pub fn log_hot_applied(mut reader: MessageReader<HotApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "hot_applied",
            target = ?msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "HoT 治疗结算"
        );
    }
}

// ==================== 回合事件日志 ====================

/// 监听 TurnStarted，输出回合开始日志
pub fn log_turn_started(mut reader: MessageReader<TurnStarted>) {
    for _msg in reader.read() {
        bevy::log::info!(
            target: "turn",
            event = "turn_started",
            "回合开始"
        );
    }
}

/// 监听 TurnEnded，输出回合结束日志
pub fn log_turn_ended(mut reader: MessageReader<TurnEnded>) {
    for _msg in reader.read() {
        bevy::log::info!(
            target: "turn",
            event = "turn_ended",
            "回合结束"
        );
    }
}

// ==================== 配置加载日志 ====================

/// 监听 ConfigLoaded，输出配置加载日志
pub fn log_config_loaded(mut reader: MessageReader<ConfigLoaded>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "config",
            event = "config_loaded",
            config_type = %msg.config_type,
            id = %msg.id,
            "配置已加载"
        );
    }
}

// ==================== Buff 事件日志 ====================

/// 监听 BuffApplied，输出 Buff 施加日志
pub fn log_buff_applied(mut reader: MessageReader<BuffApplied>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "buff_applied",
            target = ?msg.target,
            target_name = %msg.target_name,
            buff_id = %msg.buff_id,
            source = ?msg.source,
            remaining_turns = msg.remaining_turns,
            "Buff 已施加"
        );
    }
}

/// 监听 BuffRemoved，输出 Buff 移除日志
pub fn log_buff_removed(mut reader: MessageReader<BuffRemoved>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "buff_removed",
            target = ?msg.target,
            target_name = %msg.target_name,
            buff_id = %msg.buff_id,
            source = ?msg.source,
            "Buff 已移除"
        );
    }
}

/// 监听 BuffExpired，输出 Buff 过期日志
pub fn log_buff_expired(mut reader: MessageReader<BuffExpired>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "buff",
            event = "buff_expired",
            target = ?msg.target,
            target_name = %msg.target_name,
            buff_id = %msg.buff_id,
            "Buff 已过期"
        );
    }
}

// ==================== 技能事件日志 ====================

/// 监听 SkillActivated，输出技能激活日志
pub fn log_skill_activated(mut reader: MessageReader<SkillActivated>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "skill",
            event = "skill_activated",
            caster = ?msg.caster,
            caster_name = %msg.caster_name,
            skill_id = %msg.skill_id,
            target = ?msg.target,
            target_name = %msg.target_name,
            "技能激活"
        );
    }
}

// ==================== 关卡事件日志 ====================

/// 监听 LevelCompletedEvent，输出关卡完成日志
pub fn log_level_completed(mut reader: MessageReader<LevelCompletedEvent>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "campaign",
            event = "level_completed",
            level_id = %msg.level_id,
            success = msg.success,
            "关卡完成"
        );
    }
}

// ==================== 装备事件日志 ====================

/// 监听 EquipmentEquipped，输出装备穿戴日志
pub fn log_equipment_equipped(mut reader: MessageReader<EquipmentEquipped>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "equipment",
            event = "equipment_equipped",
            target = ?msg.target,
            target_name = %msg.target_name,
            equipment_id = %msg.equipment_id,
            "装备已穿戴"
        );
    }
}

/// 监听 EquipmentUnequipped，输出装备脱卸日志
pub fn log_equipment_unequipped(mut reader: MessageReader<EquipmentUnequipped>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "equipment",
            event = "equipment_unequipped",
            target = ?msg.target,
            target_name = %msg.target_name,
            equipment_id = %msg.equipment_id,
            "装备已脱卸"
        );
    }
}

// ==================== 物品事件日志 ====================

/// 监听 ItemUsed，输出物品使用日志
pub fn log_item_used(mut reader: MessageReader<ItemUsed>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "inventory",
            event = "item_used",
            user = ?msg.user,
            user_name = %msg.user_name,
            item_id = %msg.item_id,
            target = ?msg.target,
            "物品已使用"
        );
    }
}

/// 监听 ItemTransferred，输出物品转移日志
pub fn log_item_transferred(mut reader: MessageReader<ItemTransferred>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "inventory",
            event = "item_transferred",
            item_id = %msg.item_id,
            amount = msg.amount,
            from_container = %msg.from_container,
            to_container = %msg.to_container,
            "物品已转移"
        );
    }
}

// ==================== 移动事件日志 ====================

/// 监听 UnitMoved，输出单位移动日志
pub fn log_unit_moved(mut reader: MessageReader<UnitMoved>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "character",
            event = "unit_moved",
            entity = ?msg.entity,
            unit_name = %msg.unit_name,
            from_x = msg.from.x,
            from_y = msg.from.y,
            to_x = msg.to.x,
            to_y = msg.to.y,
            "单位已移动"
        );
    }
}

// ==================== 快照事件日志 ====================

/// 监听 SnapshotCreated，输出快照创建日志
pub fn log_snapshot_created(mut reader: MessageReader<SnapshotCreated>) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "core",
            event = "snapshot_created",
            snapshot_id = %msg.snapshot_id,
            entity_count = msg.entity_count,
            "场景快照已创建"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle::{CharacterDied, DamageApplied, HealApplied};
    use crate::core::character::Faction;
    use bevy::prelude::*;

    #[test]
    fn log_damage_applied_输出正确格式() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageApplied>()
            .add_systems(Update, log_damage_applied);

        let attacker = app.world_mut().spawn_empty().id();
        let target = app.world_mut().spawn_empty().id();

        app.world_mut().write_message(DamageApplied {
            target,
            target_name: "哥布林".to_string(),
            target_faction: Faction::Enemy,
            attacker,
            attacker_name: "战士".to_string(),
            attacker_faction: Faction::Player,
            amount: 15,
            is_skill: false,
            terrain_label: "平原".to_string(),
            target_coord: IVec2::new(3, 4),
            breakdown: None,
        });

        app.update();
    }

    #[test]
    fn log_heal_applied_输出正确格式() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<HealApplied>()
            .add_systems(Update, log_heal_applied);

        let target = app.world_mut().spawn_empty().id();

        app.world_mut().write_message(HealApplied {
            target,
            target_name: "战士".to_string(),
            amount: 10,
        });

        app.update();
    }

    #[test]
    fn log_character_died_输出正确格式() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CharacterDied>()
            .add_systems(Update, log_character_died);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().write_message(CharacterDied {
            entity,
            name: "哥布林".to_string(),
            faction: Faction::Enemy,
        });

        app.update();
    }
}
