//! TabPanel Factory — TabPanel 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 TabPanel。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;
use bevy::ui::widget::Button;

use super::components::{TabButton, TabPanelState};
use crate::ui::primitives::button::{
    components::{ButtonState, ButtonVariant},
    factory::spawn_button,
};
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 TabPanel UI 节点
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌和间距令牌）
/// - `tabs`: 标签名称列表
/// - `default_index`: 默认激活的标签索引
///
/// # 返回
/// TabPanel 容器实体的 Entity。内部结构：
/// ```text
/// TabPanel (Node, flex: column)
///   ├── TabBar (Node, flex: row)
///   │   ├── TabButton (ButtonVariant::Primary, 带 TabButton 组件)
///   │   ├── TabButton (ButtonVariant::Ghost, 带 TabButton 组件)
///   │   └── ...
///   └── TabContent (Panel(Basic), 标签内容占位容器)
/// ```
///
/// # 用法
/// ```ignore
/// let panel = spawn_tab_panel(
///     &mut commands, &theme,
///     &["Stats", "Skills", "Equipment"], 0,
/// );
/// commands.entity(panel).with_children(|parent| {
///     parent.spawn(Text::new("Content for active tab"));
/// });
/// ```
pub fn spawn_tab_panel(
    commands: &mut Commands,
    theme: &Theme,
    tabs: &[&str],
    default_index: usize,
) -> Entity {
    // 创建标签按钮栏（水平排列）
    let tab_bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(theme.spacing.xs),
                ..default()
            },
            Name::new("TabBar"),
        ))
        .id();

    // 创建标签按钮并设置父级为 tab_bar
    for (i, tab_label) in tabs.iter().enumerate() {
        let is_active = i == default_index;
        let variant = if is_active {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        };
        let btn = spawn_button(commands, theme, *tab_label, variant);
        commands.entity(btn).insert(TabButton { index: i });
        commands.entity(btn).set_parent_in_place(tab_bar);
    }

    // 创建标签内容区域
    let content_area = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing.md)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
                ..default()
            },
            BackgroundColor(theme.colors.surface_primary),
            BorderColor::all(theme.colors.border_default),
            Name::new("TabContent"),
        ))
        .id();

    // 创建主容器
    let container = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.spacing.sm),
                ..default()
            },
            TabPanelState {
                active_tab: default_index,
                tab_count: tabs.len(),
            },
            Name::new("TabPanel"),
        ))
        .id();

    // 设置父子关系
    commands.entity(tab_bar).set_parent_in_place(container);
    commands.entity(content_area).set_parent_in_place(container);

    container
}
