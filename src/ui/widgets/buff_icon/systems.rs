//! BuffIcon 更新系统
//!
//! 每帧同步 BuffIconState 到子控件（ProgressBar）的状态。
//! 由 BuffIconPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::primitives::progress_bar::components::ProgressBarState;

use super::components::BuffIconState;

/// BuffIcon 更新系统
///
/// 每帧对每个 BuffIcon 实体：
/// 1. 读取 BuffIconState（remaining_turns, max_turns）
/// 2. 遍历子实体，更新 ProgressBar 的 current/maximum
///
/// 进度条的视觉更新由 progress_bar_update_system 负责（它读取
/// ProgressBarState 并更新填充条宽度和标签文本），本系统仅修改数值。
pub fn buff_icon_update_system(
    parent_query: Query<(&BuffIconState, &Children), Changed<BuffIconState>>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
) {
    for (state, children) in parent_query.iter() {
        for child in children.iter() {
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                pb_state.current = state.remaining_turns as f32;
                pb_state.maximum = state.max_turns as f32;
            }
        }
    }
}
