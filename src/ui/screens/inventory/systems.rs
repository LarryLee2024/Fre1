//! 背包界面系统 — 通过 UiCommand 路由处理按钮点击
//!
//! 使用 ButtonClicked 触发 Observer 和 Commands::trigger
//! 将 InventoryGridAction 映射到领域命令（方案A）。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::widgets::inventory_grid::components::InventoryGridAction;

/// Observer：处理背包界面按钮点击，映射到 UiCommand
///
/// 当原语层的 `button_interaction_system` 通过 Commands::trigger 触发
/// `ButtonClicked` 事件时，检查按钮实体是否携带 `InventoryGridAction` 组件
/// 并分发对应的 UiCommand。
pub fn on_inventory_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&InventoryGridAction>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // 非背包按钮，忽略
        return;
    };

    let command = match action {
        InventoryGridAction::Close => UiCommand::CloseScreen,
    };

    info!(target: "ui", "[Inventory] 命令映射: {:?}", command);
    commands.trigger(command);
}
