//! MainMenuScreen systems — button click handling via UiCommand routing
//!
//! Replaces the old Observer-only pattern with a dual approach:
//! 1. ButtonClicked trigger observer maps MenuAction directly to UiCommand（方案A）
//! 2. UiAction trigger observer handles generic actions (keyboard shortcuts, etc.)

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::{UiAction, UiCommand};
use crate::ui::navigation::ScreenType;
use crate::ui::primitives::button::events::ButtonClicked;

use super::MenuAction;

/// Observer: 处理主菜单按钮点击，映射为 UiCommand
///
/// 当 primitives 层的 `button_interaction_system` 通过 Commands::trigger 触发
/// `ButtonClicked` 事件时，检查按钮实体是否挂载了 `MenuAction` 组件，
/// 匹配后通过 Commands::trigger 发送对应的领域命令（UiCommand）。
pub fn on_main_menu_button_handler(
    on: On<ButtonClicked>,
    query: Query<&MenuAction>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // 不是主菜单按钮，忽略
        return;
    };

    let command = match action {
        MenuAction::NewGame => UiCommand::NewGame,
        MenuAction::LoadGame => UiCommand::OpenScreen(ScreenType::SaveLoad),
        MenuAction::Settings => UiCommand::OpenScreen(ScreenType::Settings),
    };

    info!(target: "ui", "[MainMenu] 命令映射: {:?}", command);
    commands.trigger(command);
}

/// Observer: 处理主菜单 UiAction 事件
///
/// 接收通过 Commands::trigger 发射的 UiAction 事件，处理通用的 UI 行为
/// （如从键盘输入触发的行为）。Click 事件有专门的 ButtonClicked Observer 处理
/// （带实体上下文），此处处理其他行为。当前为 MVP 实现，仅记录日志。
pub fn on_main_menu_action(
    on: On<UiAction>,
) {
    match on.event() {
        // Click 由专门的 ButtonClicked Observer 处理（见 on_main_menu_button_handler）
        UiAction::Click => {
            trace!(target: "ui", "[MainMenu] UiAction::Click 收到（由 ButtonClicked Observer 处理）");
        }
        other => {
            info!(target: "ui", "[MainMenu] UiAction: {:?}", other);
        }
    }
}
