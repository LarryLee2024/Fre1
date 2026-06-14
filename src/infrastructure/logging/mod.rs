/// 日志模块：领域事件驱动的日志架构
/// 业务代码触发 DomainEvent → LogObserver 监听 → 输出 TracingLog
pub mod events;
mod observer;

use bevy::prelude::*;

/// 日志插件：注册 LogObserver 系统
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        // 注册所有日志相关 Message
        app.add_message::<events::ConfigLoaded>()
            .add_message::<events::BuffApplied>()
            .add_message::<events::BuffRemoved>()
            .add_message::<events::BuffExpired>()
            .add_message::<events::SkillActivated>()
            .add_message::<events::LevelCompletedEvent>()
            .add_message::<events::EquipmentEquipped>()
            .add_message::<events::EquipmentUnequipped>()
            .add_message::<events::ItemUsed>()
            .add_message::<events::ItemTransferred>()
            .add_message::<events::UnitMoved>()
            .add_message::<events::SnapshotCreated>()
            // 注册 LogObserver 系统
            .add_systems(
                Update,
                (
                    // 战斗事件（监听已有 Message）
                    observer::log_damage_applied,
                    observer::log_heal_applied,
                    observer::log_character_died,
                    observer::log_stun_applied,
                    observer::log_dot_applied,
                    observer::log_hot_applied,
                    // 回合事件（监听已有 Message）
                    observer::log_turn_started,
                    observer::log_turn_ended,
                    // 新增日志事件
                    observer::log_config_loaded,
                    observer::log_buff_applied,
                    observer::log_buff_removed,
                    observer::log_buff_expired,
                    observer::log_skill_activated,
                    observer::log_level_completed,
                    observer::log_equipment_equipped,
                    observer::log_equipment_unequipped,
                    observer::log_item_used,
                    observer::log_item_transferred,
                    observer::log_unit_moved,
                    observer::log_snapshot_created,
                ),
            );
    }
}
