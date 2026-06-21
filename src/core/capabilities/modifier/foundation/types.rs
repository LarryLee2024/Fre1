use bevy::prelude::Reflect;

pub use crate::shared::ids::ModifierInstanceId;

use serde::{Deserialize, Serialize};

/// 修改器运算类型。
///
/// 决定 Modifier 在聚合管线中的处理阶段：
/// - Add → CalcStage::Add
/// - Multiply → CalcStage::Multiply
/// - Override → CalcStage::Override
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ModifierOp {
    /// 加法：final += magnitude
    Add,
    /// 乘法：final *= (1 + magnitude) 或连乘
    Multiply,
    /// 覆盖：直接替换为 magnitude（最高优先级的 Override 生效）
    Override,
}

/// 修改器执行优先级（越小越优先）。
pub type ModifierPriority = u8;

/// 修改器来源类型。
///
/// 用于追溯 Modifier 的施加来源，支持 UI 提示和平衡分析。
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum ModifierSourceType {
    /// 来自 Buff/Debuff
    Buff,
    /// 来自装备
    Equipment,
    /// 来主动能
    Ability,
    /// 来自被动天赋
    Passive,
    /// 来自环境效果（地形、天气等）
    Environmental,
    /// 来自消耗品/道具
    Item,
    /// 来自成长系统（等级、突破等）
    Progression,
    /// 自定义来源，字符串标识
    Custom(String),
}
