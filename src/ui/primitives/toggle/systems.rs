//! Toggle 交互系统
//!
//! 检测 Toggle 指示器的按钮点击，切换 ToggleState.checked 状态，
//! 并更新指示器的背景色（checked = accent_primary, unchecked = surface_secondary）。
//!
//! 使用 Bevy 内置的 Interaction 组件 + Changed 筛选，
//! 确保每次点击只触发一次切换（不反复触发）。

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::ui::Interaction;

use super::components::{ToggleIndicator, ToggleState};
use crate::ui::theme::Theme;

/// Toggle 交互更新系统
///
/// 每帧检测 ToggleIndicator 实体的 Interaction 变化，
/// 当检测到 Pressed（刚按下）时切换父级 Toggle 的 checked 状态，
/// 并同步更新指示器颜色。
pub fn toggle_interaction_system(
    theme: Res<Theme>,
    mut indicators: Query<
        (&ChildOf, &Interaction, &mut BackgroundColor),
        (With<ToggleIndicator>, Changed<Interaction>),
    >,
    mut toggles: Query<&mut ToggleState>,
) {
    for (parent, interaction, mut bg_color) in &mut indicators {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Ok(mut state) = toggles.get_mut(parent.get()) {
            if !state.enabled {
                continue;
            }

            state.checked = !state.checked;

            if state.checked {
                *bg_color = BackgroundColor(theme.colors.accent_primary);
            } else {
                *bg_color = BackgroundColor(theme.colors.surface_secondary);
            }
        }
    }
}
