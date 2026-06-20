//! Modal Factory — 模态框的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 Modal。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;
use bevy::ui::widget::Button;

use super::components::{ModalButtonRole, ModalState, ModalVariant};
use crate::infra::localization::LocalizedText;
use crate::infra::localization::generated::loc;
use crate::ui::Theme;
use crate::ui::primitives::button::components::{ButtonInteraction, ButtonState, ButtonVariant};

/// 工厂函数：生成一个完整配置的模态框 UI 节点树
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `variant`: 模态框样式变体
/// - `title`: 模态框标题
/// - `message`: 模态框正文消息
///
/// # 返回
/// 模态框浮层实体的 Entity
///
/// # 结构
/// 创建以下节点树：
/// ```text
/// Overlay (全屏暗色半透明背景, ModalState)
///   └── Card (居中白色卡片)
///         ├── Title (标题文本)
///         ├── Message (正文文本)
///         └── ButtonRow (按钮行，Alert/Confirm 变体)
///               ├── Cancel/OK 按钮 (依变体)
///               └── Confirm 按钮 (Confirm 变体)
/// ```
///
/// # 用法
/// ```ignore
/// let modal = spawn_modal(
///     &mut commands, &theme,
///     ModalVariant::Confirm,
///     "确认操作",
///     "您确定要执行此操作吗？",
/// );
/// ```
pub fn spawn_modal(
    commands: &mut Commands,
    theme: &Theme,
    variant: ModalVariant,
    title: impl Into<String>,
    message: impl Into<String>,
) -> Entity {
    let title_str: String = title.into();
    let message_str: String = message.into();

    // 遮罩层颜色：全屏半透明暗色背景
    let overlay_color = Color::srgba(0.0, 0.0, 0.0, 0.6);

    // 卡片颜色
    let card_bg = theme.colors.surface_primary;
    let card_border = theme.colors.border_default;
    let card_radius = Val::Px(theme.spacing.border_radius_lg);

    // 文本颜色
    let title_color = theme.colors.text_primary;
    let message_color = theme.colors.text_secondary;

    commands
        .spawn((
            // 全屏遮罩层
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(overlay_color),
            ModalState {
                variant,
                title: title_str.clone(),
                message: message_str.clone(),
                open: true,
            },
            Name::new(format!("ModalOverlay({})", title_str)),
        ))
        .with_children(|parent| {
            // 居中卡片容器
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(theme.spacing.lg)),
                        row_gap: Val::Px(theme.spacing.md),
                        min_width: Val::Px(300.0),
                        max_width: Val::Px(500.0),
                        border_radius: BorderRadius::all(card_radius),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(card_bg),
                    BorderColor::all(card_border),
                    Name::new("ModalCard"),
                ))
                .with_children(|card| {
                    // 标题文本（使用 heading 字号）
                    card.spawn((
                        Text::new(title_str.clone()),
                        TextFont {
                            font_size: FontSize::Px(theme.typography.size_heading),
                            ..default()
                        },
                        TextColor(title_color),
                        Name::new("ModalTitle"),
                    ));

                    // 正文消息文本（使用 body 字号）
                    card.spawn((
                        Text::new(message_str.clone()),
                        TextFont {
                            font_size: FontSize::Px(theme.typography.size_body),
                            ..default()
                        },
                        TextColor(message_color),
                        Name::new("ModalMessage"),
                    ));

                    // 按钮行（仅 Alert 和 Confirm 变体有默认按钮）
                    let has_buttons =
                        matches!(variant, ModalVariant::Alert | ModalVariant::Confirm);
                    if has_buttons {
                        card.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::End,
                                column_gap: Val::Px(theme.spacing.sm),
                                ..default()
                            },
                            Name::new("ModalButtonRow"),
                        ))
                        .with_children(|button_row| {
                            match variant {
                                ModalVariant::Alert => {
                                    // Alert：单个"确定"按钮（Primary 风格）
                                    button_row
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                padding: UiRect::axes(
                                                    Val::Px(theme.spacing.md),
                                                    Val::Px(theme.spacing.sm),
                                                ),
                                                min_height: Val::Px(theme.spacing.button_height),
                                                ..default()
                                            },
                                            Button,
                                            BackgroundColor(theme.colors.accent_primary),
                                            BorderColor::all(Color::NONE),
                                            ButtonState {
                                                variant: ButtonVariant::Primary,
                                                disabled: false,
                                                label: String::new(),
                                            },
                                            ButtonInteraction::default(),
                                            ModalButtonRole::Cancel,
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new(""),
                                                LocalizedText::static_text(loc::core::CONFIRM),
                                                TextFont {
                                                    font_size: FontSize::Px(
                                                        theme.typography.size_body,
                                                    ),
                                                    ..default()
                                                },
                                                TextColor(theme.colors.text_primary),
                                            ));
                                        });
                                }
                                ModalVariant::Confirm => {
                                    // Confirm："取消"按钮（Secondary 风格）
                                    button_row
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                padding: UiRect::axes(
                                                    Val::Px(theme.spacing.md),
                                                    Val::Px(theme.spacing.sm),
                                                ),
                                                min_height: Val::Px(theme.spacing.button_height),
                                                ..default()
                                            },
                                            Button,
                                            BackgroundColor(theme.colors.surface_secondary),
                                            BorderColor::all(theme.colors.border_default),
                                            ButtonState {
                                                variant: ButtonVariant::Secondary,
                                                disabled: false,
                                                label: String::new(),
                                            },
                                            ButtonInteraction::default(),
                                            ModalButtonRole::Cancel,
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new(""),
                                                LocalizedText::static_text(loc::core::CANCEL),
                                                TextFont {
                                                    font_size: FontSize::Px(
                                                        theme.typography.size_body,
                                                    ),
                                                    ..default()
                                                },
                                                TextColor(theme.colors.text_primary),
                                            ));
                                        });

                                    // "确认"按钮（Primary 风格）
                                    button_row
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                padding: UiRect::axes(
                                                    Val::Px(theme.spacing.md),
                                                    Val::Px(theme.spacing.sm),
                                                ),
                                                min_height: Val::Px(theme.spacing.button_height),
                                                ..default()
                                            },
                                            Button,
                                            BackgroundColor(theme.colors.accent_primary),
                                            BorderColor::all(Color::NONE),
                                            ButtonState {
                                                variant: ButtonVariant::Primary,
                                                disabled: false,
                                                label: String::new(),
                                            },
                                            ButtonInteraction::default(),
                                            ModalButtonRole::Confirm,
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new(""),
                                                LocalizedText::static_text(loc::core::CONFIRM),
                                                TextFont {
                                                    font_size: FontSize::Px(
                                                        theme.typography.size_body,
                                                    ),
                                                    ..default()
                                                },
                                                TextColor(theme.colors.text_primary),
                                            ));
                                        });
                                }
                                // Custom 变体无默认按钮，由调用方自主添加
                                ModalVariant::Custom => {}
                            }
                        });
                    }
                });
        })
        .id()
}
