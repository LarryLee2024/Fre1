//! MainMenuScreen systems — button click handling via Observer pattern
//!
//! Observers listen for `ButtonClicked` events triggered by the primitives
//! button system, then match on `MenuAction` to determine the intended action.

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::primitives::button::events::ButtonClicked;

use super::MenuAction;

/// Observer: 处理主菜单按钮点击
///
/// 当 primitives 层的 `button_interaction_system` 触发 `ButtonClicked` 事件时，
/// 检查按钮实体是否挂载了 `MenuAction` 组件，匹配后执行对应动作。
/// 当前仅记录日志，后续由 domain 系统接管。
pub fn on_main_menu_button_clicked(on: On<ButtonClicked>, query: Query<&MenuAction>) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // 不是主菜单按钮，忽略
        return;
    };

    match action {
        MenuAction::NewGame => {
            info!("[MainMenu] New Game clicked");
        }
        MenuAction::LoadGame => {
            info!("[MainMenu] Load Game clicked");
        }
        MenuAction::Settings => {
            info!("[MainMenu] Settings clicked");
        }
    }
}
