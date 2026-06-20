//! SkillSlot 更新系统
//!
//! 每帧同步 SkillSlotState 到子控件（ProgressBar、Button）的状态。
//! 由 SkillSlotPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::primitives::button::components::ButtonState;
use crate::ui::primitives::progress_bar::components::ProgressBarState;

use super::components::SkillSlotState;

/// 技能槽更新系统
///
/// 每帧对每个 SkillSlot 实体：
/// 1. 读取 SkillSlotState（cooldown_current, cooldown_max, is_ready）
/// 2. 遍历子实体，更新 ProgressBar 的 current/maximum
/// 3. 遍历子实体，更新 Button 的 disabled 状态
///
/// 进度条的视觉更新由 progress_bar_update_system 负责（它读取
/// ProgressBarState 并更新填充条宽度和标签文本），本系统仅修改数值。
pub fn skill_slot_update_system(
    parent_query: Query<(&SkillSlotState, &Children)>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
    mut button_query: Query<&mut ButtonState>,
) {
    for (slot_state, children) in parent_query.iter() {
        for child in children.iter() {
            // 更新子 ProgressBar 的当前值和最大值
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                pb_state.current = slot_state.cooldown_current as f32;
                pb_state.maximum = slot_state.cooldown_max as f32;
            }

            // 更新子 Button 的禁用状态
            if let Ok(mut btn_state) = button_query.get_mut(child) {
                btn_state.disabled = !slot_state.is_ready;
            }
        }
    }
}
