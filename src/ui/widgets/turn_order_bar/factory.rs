//! TurnOrderBar Factory — 行动顺序栏控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

use super::components::{TurnOrderBar, TurnOrderEntry};

/// 工厂函数：生成一个完整的行动顺序栏控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Group) — "TurnOrderBar"
///   ├── Node — "TurnEntry(Aelric)"
///   │   └── Text ("Aelric", Body, primary color)
///   ├── Node — "TurnEntry(Mira)"
///   │   └── Text ("Mira", Body, secondary color)
///   ├── Node — "TurnEntry(Torvin)"
///   │   └── Text ("Torvin", Body, secondary color)
///   ├── Node — "TurnEntry(Kaelen)"
///   │   └── Text ("Kaelen", Body, secondary color)
///   └── Node — "TurnEntry(Shadow)"
///       └── Text ("Shadow", Body, secondary color)
/// ```
///
/// MVP 阶段使用静态占位数据。当前行动单位（第一个）高亮显示，
/// 其余单位用次要颜色。
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
///
/// # 返回
/// 行动顺序栏容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let bar = spawn_turn_order_bar(&mut commands, &asset_server, &theme);
/// ```
pub fn spawn_turn_order_bar(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
) -> Entity {
    // ── 1. Container panel (Group variant) ──
    // Group 提供带内边距和背景的表面
    let container = spawn_panel(commands, theme, PanelVariant::Group);

    // 覆盖容器布局为横向排列，居中对齐
    commands.entity(container).insert((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(theme.spacing.md),
            padding: UiRect::all(Val::Px(theme.spacing.sm)),
            border_radius: BorderRadius::all(Val::Px(theme.spacing.border_radius_sm)),
            ..default()
        },
        TurnOrderBar,
        Name::new("TurnOrderBar"),
    ));

    // ── 2. Placeholder entries (MVP static data) ──
    // 第一个角色为当前行动单位，其余为等待单位
    let entries = [
        ("Aelric", true),
        ("Mira", false),
        ("Torvin", false),
        ("Kaelen", false),
        ("Shadow", false),
    ];

    for (name, is_active) in entries {
        // 条目容器：用于挂载 TurnOrderEntry 组件，未来可扩展为带图标/背景的条目卡片
        let entry_container = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(theme.spacing.xs), Val::Px(theme.spacing.xs)),
                    ..default()
                },
                TurnOrderEntry {
                    unit_name: name.to_string(),
                    is_active,
                },
                Name::new(format!("TurnEntry({})", name)),
            ))
            .id();

        // 单位名称文本
        let name_text = spawn_text(commands, asset_server, theme, name, TextVariant::Body);

        // 根据是否当前行动单位设置文字颜色
        let text_color = if is_active {
            theme.colors.text_primary
        } else {
            theme.colors.text_secondary
        };
        commands.entity(name_text).insert(TextColor(text_color));
        commands
            .entity(name_text)
            .set_parent_in_place(entry_container);

        commands
            .entity(entry_container)
            .set_parent_in_place(container);
    }

    container
}
