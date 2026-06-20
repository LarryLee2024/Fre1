//! InventoryItemRow 更新系统
//!
//! 每帧同步 InventoryItemRowState 到子文本控件的内容。
//! 由 InventoryItemRowPlugin 注册到 Update 调度。

use bevy::prelude::*;

use super::components::{InventoryItemRowLabel, InventoryItemRowState};

/// 物品行更新系统
///
/// 检测 InventoryItemRowState 的变化：
/// 1. 读取 InventoryItemRowState（item_name, quantity）
/// 2. 遍历子实体，通过 InventoryItemRowLabel 标记区分名称和数量文本
/// 3. 更新对应 Text 组件的内容
pub fn inventory_item_row_update_system(
    mut query: Query<(&InventoryItemRowState, &Children), Changed<InventoryItemRowState>>,
    mut text_query: Query<(&InventoryItemRowLabel, &mut Text)>,
) {
    for (state, children) in query.iter_mut() {
        for child in children.iter() {
            if let Ok((label, mut text)) = text_query.get_mut(child) {
                match label {
                    InventoryItemRowLabel::Name => {
                        text.0 = state.item_name.clone();
                    }
                    InventoryItemRowLabel::Quantity => {
                        text.0 = format!("x{}", state.quantity);
                    }
                }
            }
        }
    }
}
