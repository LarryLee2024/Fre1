//! InventoryGrid 组件 — InventoryGrid 有机体的类型定义
//!
//! 定义 InventoryGrid 标记组件和 InventoryGridAction 枚举，
//! 用于识别网格中的交互元素。

use bevy::prelude::*;

/// InventoryGrid 标记组件
///
/// 标识 InventoryGrid Widget 的根实体。
/// 用于清理和基于查询的定位。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct InventoryGrid;

/// 可从 InventoryGrid 按钮触发的操作
///
/// 作为 Component 挂载到交互子实体上。
/// Observer 查询此组件来确定哪个按钮被点击。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum InventoryGridAction {
    /// 关闭背包界面
    Close,
}
