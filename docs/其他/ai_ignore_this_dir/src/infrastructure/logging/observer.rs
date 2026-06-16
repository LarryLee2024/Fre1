/// LogObserver：监听领域事件，输出结构化 tracing 日志
///
/// 遵循「日志是领域事件的消费者」原则（宪法 §14.8.1）。
/// 所有业务领域事件从 shared::event 导入，基础设施事件从 super::events 导入。
use bevy::ecs::message::MessageReader;
use tracing::info;

use crate::shared::event::battle;
use crate::shared::event::buff;
use crate::shared::event::campaign;
use crate::shared::event::character;
use crate::shared::event::equipment;
use crate::shared::event::inventory;
use crate::shared::event::skill;
use crate::shared::event::turn;

use crate::shared::event::infra::{ConfigLoaded, SnapshotCreated};

// ==================== 战斗事件日志 ====================

/// 监听 DamageDealt，输出伤害日志
pub fn log_damage_dealt(mut reader: MessageReader<battle::DamageDealt>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "DamageDealt",
            attacker = %msg.attacker,
            attacker_name = %msg.attacker_name,
            target = %msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            is_skill = msg.is_skill,
            is_critical = msg.is_critical,
            "伤害结算"
        );
    }
}

/// 监听 HealApplied，输出治疗日志
pub fn log_heal_applied(mut reader: MessageReader<battle::HealApplied>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "HealApplied",
            target = %msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            source = ?msg.source,
            "治疗结算"
        );
    }
}

/// 监听 CharacterDied，输出死亡日志
pub fn log_character_died(mut reader: MessageReader<battle::CharacterDied>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "CharacterDied",
            unit_id = %msg.unit_id,
            name = %msg.name,
            faction = %msg.faction,
            "单位已死亡"
        );
    }
}

/// 监听 StunApplied，输出晕眩日志
pub fn log_stun_applied(mut reader: MessageReader<battle::StunApplied>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "StunApplied",
            target = %msg.target,
            target_name = %msg.target_name,
            duration = msg.duration,
            "晕眩施加"
        );
    }
}

/// 监听 DotApplied，输出 DoT 日志
pub fn log_dot_applied(mut reader: MessageReader<battle::DotApplied>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "DotApplied",
            target = %msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "DoT 伤害结算"
        );
    }
}

/// 监听 HotApplied，输出 HoT 日志
pub fn log_hot_applied(mut reader: MessageReader<battle::HotApplied>) {
    for msg in reader.read() {
        info!(
            target: "battle",
            event = "HotApplied",
            target = %msg.target,
            target_name = %msg.target_name,
            amount = msg.amount,
            "HoT 治疗结算"
        );
    }
}

// ==================== 回合事件日志 ====================

/// 监听 TurnStarted，输出回合开始日志
pub fn log_turn_started(mut reader: MessageReader<turn::TurnStarted>) {
    for msg in reader.read() {
        info!(
            target: "turn",
            event = "TurnStarted",
            turn_number = msg.turn_number,
            faction = %msg.faction,
            "回合开始"
        );
    }
}

/// 监听 TurnEnded，输出回合结束日志
pub fn log_turn_ended(mut reader: MessageReader<turn::TurnEnded>) {
    for msg in reader.read() {
        info!(
            target: "turn",
            event = "TurnEnded",
            turn_number = msg.turn_number,
            next_faction = %msg.next_faction,
            "回合结束"
        );
    }
}

// ==================== Buff 事件日志 ====================

/// 监听 BuffApplied，输出 Buff 施加日志
pub fn log_buff_applied(mut reader: MessageReader<buff::BuffApplied>) {
    for msg in reader.read() {
        info!(
            target: "buff",
            event = "BuffApplied",
            target = %msg.target,
            target_name = %msg.target_name,
            buff_id = %msg.buff_id,
            remaining_turns = msg.remaining_turns,
            "Buff 已施加"
        );
    }
}

/// 监听 BuffRemoved，输出 Buff 移除日志
pub fn log_buff_removed(mut reader: MessageReader<buff::BuffRemoved>) {
    for msg in reader.read() {
        info!(
            target: "buff",
            event = "BuffRemoved",
            target = %msg.target,
            target_name = %msg.target_name,
            buff_id = %msg.buff_id,
            reason = ?msg.reason,
            "Buff 已移除"
        );
    }
}

// ==================== 技能事件日志 ====================

/// 监听 SkillActivated，输出技能激活日志
pub fn log_skill_activated(mut reader: MessageReader<skill::SkillActivated>) {
    for msg in reader.read() {
        info!(
            target: "skill",
            event = "SkillActivated",
            caster = %msg.caster,
            caster_name = %msg.caster_name,
            skill_id = %msg.skill_id,
            target = ?msg.target,
            target_name = ?msg.target_name,
            "技能激活"
        );
    }
}

// ==================== 战役事件日志 ====================

/// 监听 LevelCompleted，输出关卡完成日志
pub fn log_level_completed(mut reader: MessageReader<campaign::LevelCompleted>) {
    for msg in reader.read() {
        info!(
            target: "campaign",
            event = "LevelCompleted",
            level_id = %msg.level_id,
            success = msg.success,
            turns_used = msg.turns_used,
            "关卡完成"
        );
    }
}

// ==================== 装备事件日志 ====================

/// 监听 EquipmentEquipped，输出装备穿戴日志
pub fn log_equipment_equipped(mut reader: MessageReader<equipment::EquipmentEquipped>) {
    for msg in reader.read() {
        info!(
            target: "equipment",
            event = "EquipmentEquipped",
            unit_id = %msg.unit_id,
            unit_name = %msg.unit_name,
            equipment_id = %msg.equipment_id,
            "装备已穿戴"
        );
    }
}

/// 监听 EquipmentUnequipped，输出装备脱卸日志
pub fn log_equipment_unequipped(mut reader: MessageReader<equipment::EquipmentUnequipped>) {
    for msg in reader.read() {
        info!(
            target: "equipment",
            event = "EquipmentUnequipped",
            unit_id = %msg.unit_id,
            unit_name = %msg.unit_name,
            equipment_id = %msg.equipment_id,
            "装备已脱卸"
        );
    }
}

// ==================== 背包事件日志 ====================

/// 监听 ItemUsed，输出物品使用日志
pub fn log_item_used(mut reader: MessageReader<inventory::ItemUsed>) {
    for msg in reader.read() {
        info!(
            target: "inventory",
            event = "ItemUsed",
            user = %msg.user,
            user_name = %msg.user_name,
            item_id = %msg.item_id,
            target = ?msg.target,
            "物品已使用"
        );
    }
}

/// 监听 ItemTransferred，输出物品转移日志
pub fn log_item_transferred(mut reader: MessageReader<inventory::ItemTransferred>) {
    for msg in reader.read() {
        info!(
            target: "inventory",
            event = "ItemTransferred",
            item_id = %msg.item_id,
            amount = msg.amount,
            from_container = %msg.from_container,
            to_container = %msg.to_container,
            "物品已转移"
        );
    }
}

// ==================== 角色事件日志 ====================

/// 监听 UnitMoved，输出单位移动日志
pub fn log_unit_moved(mut reader: MessageReader<character::UnitMoved>) {
    for msg in reader.read() {
        info!(
            target: "character",
            event = "UnitMoved",
            unit_id = %msg.unit_id,
            unit_name = %msg.unit_name,
            from = ?msg.from,
            to = ?msg.to,
            "单位已移动"
        );
    }
}

// ==================== 基础设施事件日志 ====================

/// 监听 ConfigLoaded，输出配置加载日志
pub fn log_config_loaded(mut reader: MessageReader<ConfigLoaded>) {
    for msg in reader.read() {
        info!(
            target: "config",
            event = "ConfigLoaded",
            config_type = %msg.config_type,
            id = %msg.id,
            "配置已加载"
        );
    }
}

/// 监听 SnapshotCreated，输出快照创建日志
pub fn log_snapshot_created(mut reader: MessageReader<SnapshotCreated>) {
    for msg in reader.read() {
        info!(
            target: "core",
            event = "SnapshotCreated",
            snapshot_id = %msg.snapshot_id,
            entity_count = msg.entity_count,
            "场景快照已创建"
        );
    }
}
