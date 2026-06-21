//! ActionMenu 更新系统
//!
//! 每帧同步 ActionMenuState 到子按钮的禁用状态。
//! 由 ActionMenuPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::core::domains::combat::components::TurnQueue;
use crate::ui::application::UiCommand;
use crate::ui::primitives::button::components::ButtonState;
use crate::ui::primitives::button::events::ButtonClicked;

use super::components::{ActionMenuState, ActionType};

/// 行动菜单同步系统
///
/// 当 ActionMenuState 发生变化时：
/// 1. 遍历菜单容器的子实体
/// 2. 找到带有 ActionType 组件的按钮
/// 3. 查找 ActionMenuState.actions 中对应的 ActionMenuItem
/// 4. 根据 enabled 状态同步按钮的 disabled 状态
pub fn action_menu_sync_system(
    mut query: Query<(&ActionMenuState, &Children), Changed<ActionMenuState>>,
    mut button_query: Query<(&mut ButtonState, &ActionType)>,
) {
    for (state, children) in query.iter_mut() {
        for child in children.iter() {
            if let Ok((mut btn_state, action_type)) = button_query.get_mut(child) {
                if let Some(item) = state.actions.iter().find(|a| a.action_type == *action_type) {
                    btn_state.disabled = !item.enabled;
                }
            }
        }
    }
}

/// Observer：处理行动菜单按钮点击，映射到 UiCommand
///
/// 当 button_interaction_system 触发 ButtonClicked 事件时，
/// 检查按钮实体是否携带 ActionType 组件，映射到对应的 UiCommand。
pub fn on_action_menu_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&ActionType>,
    turn_queue: Option<Res<TurnQueue>>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action_type) = query.get(entity) else {
        return;
    };

    let current_unit_id = turn_queue
        .as_ref()
        .and_then(|q| q.current())
        .map(|entry| entry.entity.to_string())
        .unwrap_or_default();

    let command = match action_type {
        ActionType::Attack => UiCommand::Attack {
            attacker_id: current_unit_id,
            target_id: String::new(),
        },
        ActionType::Wait => UiCommand::Wait {
            unit_id: current_unit_id,
        },
        // Skill/Item 暂不映射，由后续 PR 实现
        ActionType::Skill | ActionType::Item => {
            info!(target: "ui", "[ActionMenu] {:?} 命令暂未实现", action_type);
            return;
        }
        ActionType::Defend => {
            info!(target: "ui", "[ActionMenu] Defend 命令暂未实现");
            return;
        }
    };

    info!(target: "ui", "[ActionMenu] 命令映射: {:?}", command);
    commands.trigger(command);
}
