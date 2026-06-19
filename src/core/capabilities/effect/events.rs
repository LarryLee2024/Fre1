//! Effect 领域事件
//!
//! 定义效果生命周期中的核心事件。
//! Bevy 0.18+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/effect_domain.md §6。

use bevy::prelude::*;

/// 效果成功施加时触发。
///
/// 订阅者：UI（显示 Buff 图标）、Modifier（注册修改器）、Cue（特效触发）。
#[derive(Event, Debug, Clone)]
pub struct EffectApplied {
    /// 效果实例 ID
    pub instance_id: String,
    /// EffectDef ID
    pub def_id: String,
    /// 效果分类
    pub category: String,
    /// 来源实体
    pub source_entity: String,
    /// 目标实体
    pub target_entity: String,
    /// 持续时间类型名称
    pub duration_type: String,
}

/// 效果移除时触发。
///
/// 订阅者：UI（移除 Buff 图标）、Modifier（回退修改器）、Cue（移除特效）。
#[derive(Event, Debug, Clone)]
pub struct EffectRemoved {
    /// 效果实例 ID
    pub instance_id: String,
    /// EffectDef ID
    pub def_id: String,
    /// 目标实体
    pub target_entity: String,
    /// 移除原因
    pub reason: String,
}

/// 周期效果 Tick 时触发。
///
/// 订阅者：Execution（结算 Tick 伤害/治疗）、日志、Cue（Tick 特效）。
#[derive(Event, Debug, Clone)]
pub struct EffectTicked {
    /// 效果实例 ID
    pub instance_id: String,
    /// EffectDef ID
    pub def_id: String,
    /// 目标实体
    pub target_entity: String,
    /// 已执行的 Tick 次数
    pub tick_number: u32,
    /// 总 Tick 上限
    pub total_ticks: Option<u32>,
}

/// 效果因免疫被阻止时触发。
///
/// 订阅者：Cue（显示 IMMUNE 文字）、日志。
#[derive(Event, Debug, Clone)]
pub struct EffectImmunityTriggered {
    /// EffectDef ID
    pub def_id: String,
    /// 目标实体
    pub target_entity: String,
    /// 免疫标签
    pub immune_tag: String,
}
