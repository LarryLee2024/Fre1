//! CharacterCard 更新系统
//!
//! 每帧同步 CharacterCardState 到子控件（ProgressBar）的状态。
//! 由 CharacterCardPlugin 注册到 Update 调度。

use bevy::prelude::*;

use crate::ui::primitives::progress_bar::components::ProgressBarState;

use super::components::CharacterCardState;

/// 角色卡片更新系统
///
/// 每帧对每个 CharacterCard 实体：
/// 1. 读取 CharacterCardState（hp_current, hp_max, mp_current, mp_max）
/// 2. 遍历子实体，更新 HP ProgressBar 和 MP ProgressBar 的 current/maximum
///
/// 进度条的视觉更新由 progress_bar_update_system 负责（它读取
/// ProgressBarState 并更新填充条宽度和标签文本），本系统仅修改数值。
pub fn character_card_update_system(
    parent_query: Query<(&CharacterCardState, &Children), Changed<CharacterCardState>>,
    mut progress_bar_query: Query<&mut ProgressBarState>,
) {
    for (card_state, children) in parent_query.iter() {
        for child in children.iter() {
            if let Ok(mut pb_state) = progress_bar_query.get_mut(child) {
                // Route values based on progress bar variant
                match pb_state.variant {
                    crate::ui::primitives::progress_bar::components::ProgressBarVariant::Hp => {
                        pb_state.current = card_state.hp_current;
                        pb_state.maximum = card_state.hp_max;
                    }
                    crate::ui::primitives::progress_bar::components::ProgressBarVariant::Mp => {
                        pb_state.current = card_state.mp_current;
                        pb_state.maximum = card_state.mp_max;
                    }
                    _ => {
                        // Non-HP/MP bars are not updated by this system
                    }
                }
            }
        }
    }
}
