//! InventoryScreen 系统 — 按钮点击处理

use bevy::prelude::*;

use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::widgets::inventory_grid::components::InventoryGridAction;

/// Observer: 处理关闭按钮点击
pub fn on_inventory_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&InventoryGridAction>,
) {
    let entity = on.event().entity;
    if let Ok(action) = query.get(entity) {
        match action {
            InventoryGridAction::Close => {
                info!(target: "ui", "[Inventory] Close button clicked");
            }
        }
    }
}
