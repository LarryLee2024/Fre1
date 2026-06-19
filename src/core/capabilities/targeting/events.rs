//! Targeting 领域事件
//!
//! 定义目标选择生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/targeting_domain.md §6。

use bevy::prelude::*;

/// 目标选择完成时触发。
///
/// 订阅者：Ability（继续执行流程）、UI（高亮目标）。
#[derive(Event, Debug, Clone)]
pub struct TargetSelected {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 AbilityDef ID
    pub ability_id: String,
    /// 选中目标数量
    pub target_count: u32,
    /// 首个目标实体 ID
    pub first_target: Option<String>,
}

/// 目标选择被玩家/系统修改时触发。
///
/// 订阅者：Ability（更新 Execution 目标）。
#[derive(Event, Debug, Clone)]
pub struct TargetChanged {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 AbilityDef ID
    pub ability_id: String,
    /// 原目标数量
    pub old_target_count: u32,
    /// 新目标数量
    pub new_target_count: u32,
}

/// 没有合法目标时触发。
///
/// 订阅者：Ability（技能激活失败处理）、UI（提示无合法目标）。
#[derive(Event, Debug, Clone)]
pub struct NoValidTarget {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 AbilityDef ID
    pub ability_id: String,
    /// 失败原因描述
    pub fail_reason: String,
}

/// 单个目标通过校验时触发（调试用）。
///
/// 订阅者：调试工具。
#[derive(Event, Debug, Clone)]
pub struct TargetValidated {
    /// 施法者实体
    pub entity: Entity,
    /// 目标实体
    pub target: Entity,
    /// 校验是否通过
    pub passed: bool,
    /// 校验结果描述
    pub result: String,
}
