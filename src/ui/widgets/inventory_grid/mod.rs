//! 背包物品网格有机体
//!
//! 将 Panel / Text / InventoryItemRow / Button 组合成结构化的
//! 背包网格视图，包含标题、金币显示、物品列表和关闭按钮。
//! 作为 WidgetsPlugin 的子插件注册。

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{InventoryGrid, InventoryGridAction};

/// InventoryGridPlugin — registers InventoryGrid component types
///
/// Added by WidgetsPlugin. No update systems needed as this is a
/// static layout composition of existing widgets.
pub struct InventoryGridPlugin;

impl Plugin for InventoryGridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InventoryGrid>()
            .register_type::<InventoryGridAction>();
    }
}
