//! CharacterPortrait Factory — 角色头像控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! MVP 阶段使用彩色矩形占位（无实际图片资源）。
//! 后续由 AssetServer 加载真实头像纹理。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;

use super::components::{CharacterPortrait, PortraitBorder};

/// 根据边框类型返回对应的语义边框颜色
fn border_color_for_portrait(border: PortraitBorder, theme: &Theme) -> Color {
    match border {
        PortraitBorder::Active => theme.colors.feedback_positive,
        PortraitBorder::Selected => theme.colors.border_focus,
        PortraitBorder::Inactive => theme.colors.border_default,
        PortraitBorder::None => Color::NONE,
    }
}

/// 工厂函数：生成一个完整的角色头像控件
///
/// MVP 阶段使用彩色矩形占位（无实际图片资源）。
/// 后续由 AssetServer 加载真实头像纹理。
///
/// # UI 树结构
///
/// ```text
/// Panel (Card, colored border based on PortraitBorder)
///   └── Panel (Placeholder, colored block for avatar)
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `border`: 边框类型
/// - `color`: 头像占位颜色（不同角色用不同颜色区分）
///
/// # 返回
/// CharacterPortrait 容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let portrait = spawn_character_portrait(
///     &mut commands, &theme,
///     PortraitBorder::Active,
///     Color::srgb(0.3, 0.6, 0.9),
/// );
/// ```
pub fn spawn_character_portrait(
    commands: &mut Commands,
    theme: &Theme,
    border: PortraitBorder,
    color: Color,
) -> Entity {
    let border_color = border_color_for_portrait(border, theme);

    // ── 1. Container panel (Card variant, fixed size, custom border) ──
    let container = spawn_panel(commands, theme, PanelVariant::Card);

    // Override Node for fixed portrait size (64x64) with visible 2px border,
    // compact padding, and centered content alignment.
    // Override BorderColor based on portrait border type.
    // Mount CharacterPortrait marker component and meaningful Name.
    commands.entity(container).insert((
        Node {
            width: Val::Px(64.0),
            height: Val::Px(64.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(theme.spacing.xs)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_lg)),
            ..default()
        },
        BorderColor::all(border_color),
        CharacterPortrait,
        Name::new("CharacterPortrait"),
    ));

    // ── 2. Placeholder avatar block (colored square representing the character) ──
    let placeholder = spawn_panel(
        commands,
        theme,
        PanelVariant::Placeholder {
            width: Val::Px(56.0),
            height: Val::Px(56.0),
            color,
        },
    );
    commands.entity(placeholder).set_parent_in_place(container);

    container
}
