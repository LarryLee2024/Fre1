//! ActionMenu 更新系统
//!
//! 每帧同步 ActionMenuState 到子按钮的禁用状态。
//! 由 ActionMenuPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::primitives::button::components::ButtonState;

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
