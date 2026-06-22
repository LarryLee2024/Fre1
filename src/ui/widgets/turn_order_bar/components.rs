//! TurnOrderBar 组件的类型定义
//!
//! 定义 TurnOrderBar 标记组件和 TurnOrderEntry（单个行动顺序条目组件）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// TurnOrderBar 标记组件
///
/// 挂载在 Z8 底部栏容器实体上，用于标识行动顺序栏。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct TurnOrderBar;

/// 单个行动顺序条目
///
/// 标记每个行动顺序条目（单位文本/图标），携带单位名称和是否当前行动单位。
#[derive(Component, Debug, Clone, Reflect)]
pub struct TurnOrderEntry {
    /// 单位显示名称
    pub unit_name: String,
    /// 是否为当前行动单位（高亮显示）
    pub is_active: bool,
}
