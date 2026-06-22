//! UnitSummary 组件的类型定义
//!
//! 定义 UnitSummary 标记组件用于标识 Z3 单位摘要容器。
//! MVP 阶段使用静态数据（来自 BattleHudData），无需运行时状态组件。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// UnitSummary 标记组件
///
/// 挂载在 Z3 右上区容器实体上，用于标识单位摘要控件。
/// 包含当前选中单位的名称、等级和 HP 摘要信息。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct UnitSummary;
