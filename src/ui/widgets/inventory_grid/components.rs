//! InventoryGrid 组件定义

use bevy::prelude::*;

/// InventoryGrid 标记组件
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct InventoryGrid;

/// InventoryGrid 按钮动作
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum InventoryGridAction {
    Close,
}
