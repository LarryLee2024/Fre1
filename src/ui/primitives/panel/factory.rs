//! Panel Factory — Panel 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 Panel。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::{PanelState, PanelVariant};
use crate::ui::theme::Theme;

/// 根据变体计算 Panel 背景色
fn panel_background_color(variant: PanelVariant, theme: &Theme) -> Color {
    match variant {
        PanelVariant::Basic | PanelVariant::Card | PanelVariant::Group => {
            theme.colors.surface_primary
        }
        PanelVariant::Modal => Color::srgba(0.0, 0.0, 0.0, 0.6),
        PanelVariant::Tooltip => theme.colors.surface_secondary,
        PanelVariant::List => Color::NONE,
    }
}

/// 工厂函数：生成一个完整配置的 Panel UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `variant`: Panel 样式变体
///
/// # 返回
/// Panel 容器的 Entity。调用方通过 `commands.entity(entity).with_children(...)` 添加子节点。
///
/// # 用法
/// ```ignore
/// let panel = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
/// commands.entity(panel).with_children(|parent| {
///     parent.spawn(Text::new("Content"));
/// });
/// ```
pub fn spawn_panel(commands: &mut Commands, theme: &Theme, variant: PanelVariant) -> Entity {
    let bg_color = panel_background_color(variant, theme);

    let (node, border_color) = match variant {
        PanelVariant::Basic => (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing.md)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            Color::NONE,
        ),
        PanelVariant::Card => (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing.lg)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_lg)),
                ..default()
            },
            theme.colors.border_default,
        ),
        PanelVariant::Modal => (
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(theme.spacing.lg)),
                ..default()
            },
            Color::NONE,
        ),
        PanelVariant::Tooltip => (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing.sm)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            Color::NONE,
        ),
        PanelVariant::List => (
            Node {
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            Color::NONE,
        ),
        PanelVariant::Group => (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing.md)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            Color::NONE,
        ),
    };

    let variant_name = match variant {
        PanelVariant::Basic => "Basic",
        PanelVariant::Card => "Card",
        PanelVariant::Modal => "Modal",
        PanelVariant::Tooltip => "Tooltip",
        PanelVariant::List => "List",
        PanelVariant::Group => "Group",
    };

    commands
        .spawn((
            node,
            BackgroundColor(bg_color),
            BorderColor::all(border_color),
            PanelState {
                variant,
                padded: true,
                title: None,
            },
            Name::new(format!("Panel({})", variant_name)),
        ))
        .id()
}
