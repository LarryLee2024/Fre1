//! Button Factory — 按钮的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 Button。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;
use bevy::ui::widget::Button;

use super::components::{ButtonInteraction, ButtonState, ButtonVariant};
use crate::ui::theme::Theme;

/// 根据变体和交互状态计算按钮背景色
fn button_background_color(
    variant: ButtonVariant,
    interaction: &ButtonInteraction,
    theme: &Theme,
) -> Color {
    if interaction.pressed {
        match variant {
            ButtonVariant::Primary => theme.colors.accent_pressed,
            ButtonVariant::Secondary => theme.colors.accent_pressed,
            ButtonVariant::Danger => theme.colors.surface_danger,
            ButtonVariant::Ghost => theme.colors.surface_secondary,
        }
    } else if interaction.hovered {
        match variant {
            ButtonVariant::Primary => theme.colors.accent_hover,
            ButtonVariant::Secondary => theme.colors.accent_hover,
            ButtonVariant::Danger => theme.colors.feedback_negative,
            ButtonVariant::Ghost => theme.colors.surface_secondary,
        }
    } else {
        match variant {
            ButtonVariant::Primary => theme.colors.accent_primary,
            ButtonVariant::Secondary => theme.colors.surface_secondary,
            ButtonVariant::Danger => theme.colors.feedback_negative,
            ButtonVariant::Ghost => Color::NONE,
        }
    }
}

/// 根据变体和禁用状态计算按钮文本色
fn button_text_color(_variant: ButtonVariant, disabled: bool, theme: &Theme) -> Color {
    if disabled {
        return theme.colors.text_disabled;
    }
    theme.colors.text_primary
}

/// 工厂函数：生成一个完整配置的按钮 UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `label`: 按钮文本
/// - `variant`: 按钮样式变体
///
/// # 返回
/// 按钮实体的 Entity
///
/// # 用法
/// ```ignore
/// let btn = spawn_button(&mut commands, &theme, "确认", ButtonVariant::Primary);
/// ```
pub fn spawn_button(
    commands: &mut Commands,
    theme: &Theme,
    label: impl Into<String>,
    variant: ButtonVariant,
) -> Entity {
    let label_str: String = label.into();
    let state = ButtonState {
        variant,
        disabled: false,
        label: label_str.clone(),
    };
    let bg_color = button_background_color(variant, &ButtonInteraction::default(), theme);
    let text_color = button_text_color(variant, false, theme);

    let border = match variant {
        ButtonVariant::Secondary => theme.colors.border_default,
        _ => Color::NONE,
    };

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(theme.spacing.md), Val::Px(theme.spacing.sm)),
                min_height: Val::Px(theme.spacing.button_height),
                ..default()
            },
            Button,
            BackgroundColor(bg_color),
            BorderColor::all(border),
            state,
            ButtonInteraction::default(),
            Name::new(format!("Button({})", label_str)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label_str),
                TextFont {
                    font_size: FontSize::Px(theme.typography.size_body),
                    ..default()
                },
                TextColor(text_color),
            ));
        })
        .id()
}
