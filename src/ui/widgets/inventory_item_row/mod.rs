//! 模块名: InventoryItemRow Widget — 物品行复合控件
//!
//! 组合 Panel / Text / Button 三个原子组件为一个水平物品行。
//! 每个行显示物品图标名称、数量和"使用"按钮。
//!
//! 契约:
//!   输入属性:    item_name, quantity（通过 InventoryItemRowState）
//!   输出事件:  InventoryItemAction::Use 标记在按钮实体上供 Observer 路由
//!   本地状态:      InventoryItemRowState（item_name, quantity）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{InventoryItemAction, InventoryItemRowLabel, InventoryItemRowState};
use self::systems::inventory_item_row_update_system;

/// InventoryItemRowPlugin — 注册 InventoryItemRow Widget 所需的 Component/System
pub struct InventoryItemRowPlugin;

impl Plugin for InventoryItemRowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InventoryItemRowState>()
            .register_type::<InventoryItemAction>()
            .register_type::<InventoryItemRowLabel>()
            .add_systems(Update, inventory_item_row_update_system);
    }
}
