//! List Factory — 列表的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 List。
//! 输入 Props + Theme -> 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::{ListState, ListVariant};
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 List UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌和间距令牌）
/// - `variant`: 列表排列变体
///
/// # 返回
/// 列表容器 Entity。调用方通过 `commands.entity(entity).with_children(...)` 添加子节点。
///
/// # 用法
/// ```ignore
/// let list = spawn_list(&mut commands, &theme, ListVariant::Vertical);
/// commands.entity(list).with_children(|parent| {
///     parent.spawn(Text::new("Item 1"));
///     parent.spawn(Text::new("Item 2"));
/// });
/// ```
pub fn spawn_list(commands: &mut Commands, theme: &Theme, variant: ListVariant) -> Entity {
    let (flex_direction, row_gap, column_gap, overflow) = match variant {
        ListVariant::Vertical => (
            FlexDirection::Column,
            Val::Px(theme.spacing.sm),
            Val::Px(0.0),
            Overflow::visible(),
        ),
        ListVariant::Horizontal => (
            FlexDirection::Row,
            Val::Px(0.0),
            Val::Px(theme.spacing.sm),
            Overflow::visible(),
        ),
        ListVariant::Virtual => (
            FlexDirection::Column,
            Val::Px(theme.spacing.sm),
            Val::Px(0.0),
            Overflow::clip(),
        ),
    };

    let variant_name = match variant {
        ListVariant::Vertical => "Vertical",
        ListVariant::Horizontal => "Horizontal",
        ListVariant::Virtual => "Virtual",
    };

    commands
        .spawn((
            Node {
                flex_direction,
                row_gap,
                column_gap,
                overflow,
                ..default()
            },
            ListState {
                variant,
                spacing: theme.spacing.sm,
            },
            Name::new(format!("List({})", variant_name)),
        ))
        .id()
}
