//! Button 交互系统
//!
//! 每帧检测按钮的 Interaction 状态，更新 ButtonInteraction 组件，
//! 并在检测到点击时通过 Observer 模式触发 ButtonClicked 事件
//! 和 UiAction::Click（Bevy 0.19 Commands::trigger + Observer 双通道）。
//!
//! 依赖 Bevy 内置的 Interaction 组件（UI 系统自动更新）。

use bevy::prelude::*;
use bevy::ui::Interaction;

use super::components::{ButtonInteraction, ButtonState, ButtonVariant};
use super::events::ButtonClicked;
use crate::ui::Theme;
use crate::ui::application::UiAction;

/// 根据按钮变体和交互状态计算背景色（禁用时固定为 surface_disabled）
fn resolve_bg_color(
    variant: ButtonVariant,
    disabled: bool,
    interaction: &ButtonInteraction,
    theme: &Theme,
) -> Color {
    if disabled {
        return theme.colors.surface_disabled;
    }
    let c = &theme.colors;
    if interaction.pressed {
        match variant {
            ButtonVariant::Primary => c.accent_pressed,
            ButtonVariant::Secondary => c.accent_pressed,
            ButtonVariant::Danger => c.surface_danger,
            ButtonVariant::Ghost => c.surface_secondary,
        }
    } else if interaction.hovered {
        match variant {
            ButtonVariant::Primary => c.accent_hover,
            ButtonVariant::Secondary => c.accent_hover,
            ButtonVariant::Danger => c.feedback_negative,
            ButtonVariant::Ghost => c.surface_secondary,
        }
    } else {
        match variant {
            ButtonVariant::Primary => c.accent_primary,
            ButtonVariant::Secondary => c.surface_secondary,
            ButtonVariant::Danger => c.feedback_negative,
            ButtonVariant::Ghost => Color::NONE,
        }
    }
}

/// 交互状态更新系统
///
/// 每帧对所有 Button 实体：
/// 1. 读取 Bevy 自动管理的 Interaction 组件
/// 2. 更新 ButtonInteraction（hovered / pressed / just_clicked）
/// 3. 在 just_clicked 时通过 Observer 机制触发 ButtonClicked 事件
///    并同步触发 UiAction::Click（Bevy 0.19 Commands::trigger 双通道）
/// 4. 根据状态更新 BackgroundColor
pub fn button_interaction_system(
    theme: Res<Theme>,
    mut button_query: Query<(
        Entity,
        &Interaction,
        &ButtonState,
        &mut ButtonInteraction,
        &mut BackgroundColor,
    )>,
    mut commands: Commands,
) {
    for (entity, interaction, state, mut btn_interaction, mut bg_color) in &mut button_query {
        // 禁用态：跳过交互，固定背景色
        if state.disabled {
            *bg_color = BackgroundColor(resolve_bg_color(
                state.variant,
                true,
                &btn_interaction,
                &theme,
            ));
            btn_interaction.hovered = false;
            btn_interaction.pressed = false;
            btn_interaction.just_clicked = false;
            continue;
        }

        // 追踪悬停和按压状态
        btn_interaction.hovered = *interaction == Interaction::Hovered;

        let was_pressed = btn_interaction.pressed;
        btn_interaction.pressed = *interaction == Interaction::Pressed;

        // 点击释放时设置 just_clicked（持续一帧）
        btn_interaction.just_clicked = was_pressed && !btn_interaction.pressed;

        // 触发点击事件（Bevy 0.19 Commands::trigger 双通道）
        if btn_interaction.just_clicked {
            commands.trigger(ButtonClicked { entity });
            commands.trigger(UiAction::Click);
        }

        *bg_color = BackgroundColor(resolve_bg_color(
            state.variant,
            false,
            &btn_interaction,
            &theme,
        ));
    }
}
