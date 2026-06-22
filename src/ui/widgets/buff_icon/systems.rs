//! BuffIcon 更新系统
//!
//! 每帧同步 BuffIconState 到子控件（ProgressBar）的状态。
//! Buff 图标呼吸动画系统为 Debuff 添加微弱的边框脉冲效果。
//! 由 BuffIconPlugin 注册到 Update 调度。

use bevy::color::Srgba;
use bevy::prelude::*;

use crate::ui::primitives::progress_bar::components::ProgressBarState;
use crate::ui::theme::Theme;

use super::components::{BuffIconState, BuffType};

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

/// Buff 图标呼吸动画系统
///
/// 对 Debuff 图标添加微弱的边框颜色脉冲效果（alpha 呼吸），
/// 增强视觉反馈。Buff 和 Neutral 图标不应用动画。
///
/// 使用 theme 颜色确保与当前主题一致。
pub fn buff_icon_breathing_system(
    time: Res<Time>,
    theme: Res<Theme>,
    query: Query<(Entity, &BuffIconState)>,
    mut border_query: Query<&mut BorderColor>,
) {
    // 呼吸因子：0.7 ~ 1.0，周期约 2 秒
    let breath = (time.elapsed_secs() * 3.0).sin() * 0.15 + 0.85;

    for (entity, state) in query.iter() {
        if state.buff_type != BuffType::Debuff {
            continue;
        }

        let base_color = theme.colors.feedback_negative;
        if let Ok(mut border) = border_query.get_mut(entity) {
            let srgba: Srgba = base_color.into();
            *border = BorderColor::all(Color::from(Srgba {
                red: srgba.red,
                green: srgba.green,
                blue: srgba.blue,
                alpha: breath,
            }));
        }
    }
}
