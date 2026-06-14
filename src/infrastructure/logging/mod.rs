/// 日志模块：领域事件驱动的日志架构
///
/// 业务代码触发 DomainEvent → LogObserver 监听 → 输出结构化 tracing 日志。
///
/// 业务领域事件统一从 shared::event 注册，基础设施事件保留在 events 子模块。
pub mod events;
mod observer;

use bevy::prelude::*;

use crate::shared::event::battle;
use crate::shared::event::buff;
use crate::shared::event::campaign;
use crate::shared::event::character;
use crate::shared::event::equipment;
use crate::shared::event::infra;
use crate::shared::event::inventory;
use crate::shared::event::skill;
use crate::shared::event::turn;

/// 日志插件：注册所有 LogObserver 系统
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册所有可观察 Message ──
        // 战斗
        app.add_message::<battle::DamageDealt>()
            .add_message::<battle::HealApplied>()
            .add_message::<battle::CharacterDied>()
            .add_message::<battle::StunApplied>()
            .add_message::<battle::DotApplied>()
            .add_message::<battle::HotApplied>()
            // 回合
            .add_message::<turn::TurnStarted>()
            .add_message::<turn::TurnEnded>()
            // Buff
            .add_message::<buff::BuffApplied>()
            .add_message::<buff::BuffRemoved>()
            // 技能
            .add_message::<skill::SkillActivated>()
            // 战役
            .add_message::<campaign::LevelCompleted>()
            // 装备
            .add_message::<equipment::EquipmentEquipped>()
            .add_message::<equipment::EquipmentUnequipped>()
            // 背包
            .add_message::<inventory::ItemUsed>()
            .add_message::<inventory::ItemTransferred>()
            // 角色
            .add_message::<character::UnitMoved>()
            // 基础设施
            .add_message::<infra::ConfigLoaded>()
            .add_message::<infra::SnapshotCreated>()
            // ── 注册 LogObserver 系统 ──
            .add_systems(
                Update,
                (
                    // 战斗
                    observer::log_damage_dealt,
                    observer::log_heal_applied,
                    observer::log_character_died,
                    observer::log_stun_applied,
                    observer::log_dot_applied,
                    observer::log_hot_applied,
                    // 回合
                    observer::log_turn_started,
                    observer::log_turn_ended,
                    // Buff
                    observer::log_buff_applied,
                    observer::log_buff_removed,
                    // 技能
                    observer::log_skill_activated,
                    // 战役
                    observer::log_level_completed,
                    // 装备
                    observer::log_equipment_equipped,
                    observer::log_equipment_unequipped,
                    // 背包
                    observer::log_item_used,
                    observer::log_item_transferred,
                    // 角色
                    observer::log_unit_moved,
                    // 基础设施
                    observer::log_config_loaded,
                    observer::log_snapshot_created,
                ),
            );
    }
}
