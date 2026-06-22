//! CharacterPortrait 组件的类型定义
//!
//! 定义 CharacterPortrait 标记组件和 PortraitBorder 边框类型枚举。
//! CharacterPortrait 挂载在头像容器实体上，PortraitBorder 标记边框样式状态。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// CharacterPortrait 标记组件
///
/// 标记角色头像的容器实体，供外部系统查询和更新。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct CharacterPortrait;

/// 头像边框类型
///
/// 定义角色头像在不同状态下的边框样式：
/// - `None`: 无边框
/// - `Active`: 当前行动单位（绿色/金色边框）
/// - `Inactive`: 非行动单位（灰色边框）
/// - `Selected`: 选中单位（蓝色高亮边框）
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum PortraitBorder {
    /// 无边框
    None,
    /// 当前行动单位
    Active,
    /// 非行动单位
    Inactive,
    /// 选中单位
    Selected,
}
