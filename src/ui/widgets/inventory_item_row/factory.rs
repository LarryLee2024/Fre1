//! InventoryItemRow Factory — 物品行复合控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{
    components::ButtonVariant,
    factory::spawn_localized_button,
};
use crate::ui::primitives::panel::{
    components::PanelVariant,
    factory::spawn_panel,
};
use crate::ui::primitives::text::{
    components::TextVariant,
    factory::spawn_text,
};
use crate::ui::theme::Theme;

use super::components::{InventoryItemAction, InventoryItemRowLabel, InventoryItemRowState};

/// 工厂函数：生成一个完整的物品行控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, horizontal row)
///   ├── Text (item name, Caption, primary color)
///   ├── Text ("x5", Caption, secondary color)
///   └── Button ("Use", Primary) — InventoryItemAction::Use
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `item_name`: 物品显示名称
/// - `quantity`: 物品数量
///
/// # 返回
/// 物品行容器实体的 Entity
///
/// # 用法
/// ```ignore
/// let row = spawn_inventory_item_row(
///     &mut commands, &asset_server, &theme,
///     "Health Potion", 5,
/// );
/// ```
pub fn spawn_inventory_item_row(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    item_name: impl Into<String>,
    quantity: u32,
) -> Entity {
    let name_str: String = item_name.into();
    let qty_str = format!("x{}", quantity);

    // ── 1. Container panel (Basic variant, overridden to Row layout) ──
    // spawn_panel creates a Column layout by default; we override to Row
    // for the horizontal inventory row.
    let container = spawn_panel(commands, theme, PanelVariant::Basic);

    // Override to horizontal row layout
    commands.entity(container).insert((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(theme.spacing.sm),
            padding: UiRect::axes(
                Val::Px(theme.spacing.sm),
                Val::Px(theme.spacing.xs),
            ),
            ..default()
        },
        InventoryItemRowState {
            item_name: name_str.clone(),
            quantity,
        },
        Name::new(format!("InventoryItemRow({})", name_str)),
    ));

    // ── 2. Item name text (Caption variant, primary color) ──
    let name_text = spawn_text(commands, asset_server, theme, &name_str, TextVariant::Caption);
    commands.entity(name_text).insert((
        TextColor(theme.colors.text_primary),
        InventoryItemRowLabel::Name,
    ));
    commands.entity(name_text).set_parent_in_place(container);

    // ── 3. Quantity text (Caption variant, secondary color) ──
    let qty_text = spawn_text(commands, asset_server, theme, &qty_str, TextVariant::Caption);
    commands.entity(qty_text).insert(InventoryItemRowLabel::Quantity);
    commands.entity(qty_text).set_parent_in_place(container);

    // ── 4. Use button (Primary variant) ──
    let use_btn = spawn_localized_button(commands, theme, loc::ui::USE, "Use", ButtonVariant::Primary);
    commands.entity(use_btn).insert(InventoryItemAction::Use);
    commands.entity(use_btn).set_parent_in_place(container);

    container
}
