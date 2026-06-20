//! ProgressBar Factory — 进度条的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 ProgressBar。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::Theme;
use super::components::{ProgressBarFill, ProgressBarLabel, ProgressBarState, ProgressBarVariant};

/// 根据变体计算进度条填充色
fn progress_bar_fill_color(variant: ProgressBarVariant, theme: &Theme) -> Color {
    match variant {
        ProgressBarVariant::Hp => theme.colors.feedback_positive,
        ProgressBarVariant::Mp => theme.colors.accent_primary,
        ProgressBarVariant::Xp => theme.colors.feedback_warning,
        ProgressBarVariant::Generic => theme.colors.accent_primary,
    }
}

/// 工厂函数：生成一个完整配置的进度条 UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `variant`: 进度条样式变体
/// - `current`: 当前值
/// - `maximum`: 最大值
/// - `show_label`: 是否显示变体前缀 + "current/maximum" 标签文本
/// - `height`: 进度条高度
///
/// # 返回
/// 进度条实体的 Entity
///
/// # 用法
/// ```ignore
/// let hp_bar = spawn_progress_bar(
///     &mut commands, &theme,
///     ProgressBarVariant::Hp, 80.0, 100.0, true,
///     Val::Px(theme.spacing.md),
/// );
/// ```
pub fn spawn_progress_bar(
    commands: &mut Commands,
    theme: &Theme,
    variant: ProgressBarVariant,
    current: f32,
    maximum: f32,
    show_label: bool,
    height: Val,
) -> Entity {
    let ratio = if maximum > 0.0 {
        (current / maximum).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let fill_color = progress_bar_fill_color(variant, theme);

    let prefix = match variant {
        ProgressBarVariant::Hp => "HP ",
        ProgressBarVariant::Mp => "MP ",
        ProgressBarVariant::Xp => "XP ",
        ProgressBarVariant::Generic => "",
    };
    let label_text = format!("{}{:.0}/{}", prefix, current, maximum as u32);

    let variant_name = match variant {
        ProgressBarVariant::Hp => "Hp",
        ProgressBarVariant::Mp => "Mp",
        ProgressBarVariant::Xp => "Xp",
        ProgressBarVariant::Generic => "Generic",
    };

    commands
        .spawn((
            Node {
                height,
                overflow: Overflow::clip(),
                position_type: PositionType::Relative,
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            BackgroundColor(theme.colors.surface_secondary),
            ProgressBarState {
                variant,
                current,
                maximum,
                show_label,
            },
            Name::new(format!("ProgressBar({})", variant_name)),
        ))
        .with_children(|parent| {
            // 填充条（绝对定位，宽度由 current/maximum 比例决定）
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(ratio * 100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                    ..default()
                },
                BackgroundColor(fill_color),
                ProgressBarFill,
            ));

            // 标签（绝对定位，居中显示）
            if show_label {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ))
                .with_children(|label_parent| {
                    label_parent.spawn((
                        Text::new(label_text),
                        TextFont {
                            font_size: FontSize::Px(theme.typography.size_small),
                            ..default()
                        },
                        TextColor(theme.colors.text_primary),
                        ProgressBarLabel,
                    ));
                });
            }
        })
        .id()
}
