//! Spec 领域事件
//!
//! 定义 Spec 生命周期中的四个核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/spec_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::spec::foundation::{SpecId, SpecType};

/// Spec 成功授予到实体时触发。
///
/// 订阅者：Ability（注册可激活技能列表）、UI（更新技能栏）。
#[derive(Event, Debug, Clone)]
pub struct SpecGranted {
    /// 目标实体
    pub entity: Entity,
    /// Spec 类型（ability / effect）
    pub spec_type: SpecType,
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 引用的 Def ID
    pub def_id: String,
}

/// Spec 从实体移除时触发。
///
/// 订阅者：Ability（清理关联）、UI（更新技能栏）。
#[derive(Event, Debug, Clone)]
pub struct SpecRemoved {
    /// 目标实体
    pub entity: Entity,
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 移除原因
    pub reason: SpecRemovalReason,
}

/// Spec 移除原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecRemovalReason {
    /// 手动移除
    Manual,
    /// 持续时间结束
    Expired,
    /// 被替换
    Replaced,
}

/// AbilitySpec 等级变更时触发。
///
/// 订阅者：Ability（更新能力参数）、Progression（确认升级效果）。
#[derive(Event, Debug, Clone)]
pub struct SpecLevelChanged {
    /// 目标实体
    pub entity: Entity,
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 变更前等级
    pub old_level: u8,
    /// 变更后等级
    pub new_level: u8,
}

/// EffectSpec 快照属性值时触发。
///
/// 订阅者：回放系统、回滚系统。
#[derive(Event, Debug, Clone)]
pub struct SpecSnapshotTaken {
    /// 目标实体
    pub entity: Entity,
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 快照数据（JSON-like 的键值对，具体内容由 EffectSnapshot 决定）
    pub snapshot_data: String,
}
