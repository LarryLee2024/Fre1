//! ScrollPanel Factory — ScrollPanel 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 ScrollPanel。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::{ScrollContent, ScrollPanelState};
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 ScrollPanel UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌和间距令牌）
/// - `max_height`: 容器最大可见高度（像素），超出部分可滚动
/// - `padding`: 容器内边距
///
/// # 返回
/// ScrollPanel 容器实体的 Entity。内部结构：
/// ```text
/// ScrollPanel (Node, overflow: clip, max_height)
///   └── ScrollContent (Node, 内容包装器，translate 偏移由此节点承载)
/// ```
///
/// 调用方通过 `commands.entity(entity).with_children(...)` 添加子节点，
/// 子节点会自动成为 `ScrollContent` 的兄弟节点。
///
/// # 用法
/// ```ignore
/// let panel = spawn_scroll_panel(
///     &mut commands, &theme, 300.0,
///     UiRect::all(Val::Px(theme.spacing.md)),
/// );
/// commands.entity(panel).with_children(|parent| {
///     parent.spawn(Text::new("Scrollable content"));
/// });
/// ```
pub fn spawn_scroll_panel(
    commands: &mut Commands,
    theme: &Theme,
    max_height: f32,
    padding: UiRect,
) -> Entity {
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                max_height: Val::Px(max_height),
                padding,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            BackgroundColor(theme.colors.surface_primary),
            BorderColor::all(theme.colors.border_default),
            ScrollPanelState {
                scroll_offset: 0.0,
                content_height: 0.0,
                max_height,
            },
            Name::new("ScrollPanel"),
        ))
        .with_children(|parent| {
            // 内部内容包装容器，scroll_offset 通过 translate 应用到此节点
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ScrollContent,
                Name::new("ScrollContent"),
            ));
        })
        .id()
}
