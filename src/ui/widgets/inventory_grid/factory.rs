//! InventoryGrid Factory — InventoryGrid 复合控件的唯一创建入口
//!
//! 组合 Panel / Text / InventoryItemRow / Button 四个控件为一个结构化的
//! 背包网格视图，包含标题、金币显示、物品列表和关闭按钮。

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{
    components::TextVariant,
    factory::{spawn_localized_text, spawn_text},
};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_item_row::factory::spawn_inventory_item_row;

use super::components::{InventoryGrid, InventoryGridAction};

/// 工厂函数：生成一个完整的 InventoryGrid 控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, column layout)
///   ├── Text ("Inventory", Heading)
///   ├── Text ("Gold: 100", Caption)
///   ├── InventoryItemRow ("Health Potion", x5)
///   ├── InventoryItemRow ("Mana Potion", x3)
///   ├── InventoryItemRow ("Antidote", x2)
///   ├── InventoryItemRow ("Phoenix Down", x1)
///   └── Button ("Close", Secondary) — InventoryGridAction::Close
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器
/// - `theme`: 主题 Resource
///
/// # 返回
/// 背包网格容器实体的 Entity
pub fn spawn_inventory_grid(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
) -> Entity {
    // ── 1. Container panel ──
    let container = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(container).insert((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(400.0),
            padding: UiRect::all(Val::Px(theme.spacing.lg)),
            row_gap: Val::Px(theme.spacing.sm),
            ..default()
        },
        InventoryGrid,
        Name::new("InventoryGrid"),
    ));

    // ── 2. Title ──
    let title = spawn_localized_text(
        commands,
        asset_server,
        theme,
        loc::ui::INVENTORY,
        "Inventory",
        TextVariant::Heading,
    );
    commands.entity(title).set_parent_in_place(container);

    // ── 3. Gold display ──
    // TODO[P3][Projection][2026-06-21]: Replace hardcoded "Gold: 100" with ViewModel binding
    //   - Create EconomyVm (gold: u32) in view_models/
    //   - Add EconomyVm to UiStore
    //   - Wire EconomyProjection to update gold on purchase/sell events
    //   - Create GoldDisplay widget that reads from UiStore via Dirty<EconomyVm>
    //   Completion criteria: gold display reflects actual player gold from economy domain
    let gold = spawn_text(
        commands,
        asset_server,
        theme,
        "Gold: 100",
        TextVariant::Caption,
    );
    commands.entity(gold).set_parent_in_place(container);

    // ── 4. Sample inventory item rows ──
    let items: [(&str, u32); 4] = [
        ("Health Potion", 5),
        ("Mana Potion", 3),
        ("Antidote", 2),
        ("Phoenix Down", 1),
    ];

    for (name, qty) in items {
        let row = spawn_inventory_item_row(commands, asset_server, theme, name, qty);
        commands.entity(row).set_parent_in_place(container);
    }

    // ── 5. Close button (Secondary variant) ──
    let close = spawn_localized_button(
        commands,
        theme,
        loc::ui::CLOSE,
        "Close",
        ButtonVariant::Secondary,
    );
    commands.entity(close).insert(InventoryGridAction::Close);
    commands.entity(close).set_parent_in_place(container);

    container
}
