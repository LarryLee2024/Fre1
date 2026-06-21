//! ShopPanel Factory — 商店面板有机体的唯一创建入口
//!
//! 组合 Panel / Text / TabPanel / ShopItemCard / InventoryItemRow / Button
//! 为一个结构化的商店面板，包含标题、金币显示、购买/出售标签页、
//! 商品卡片列表和关闭按钮。

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::tab_panel::factory::spawn_tab_panel;
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_item_row::factory::spawn_inventory_item_row;
use crate::ui::widgets::shop_item_card::factory::spawn_shop_item_card;

use super::components::{ShopPanel, ShopPanelAction};

/// 工厂函数：生成一个完整的 ShopPanel 控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, column layout)
///   ├── Panel (Header row)
///   │   ├── Text ("Shop", Heading)
///   │   └── Text ("Gold: 999", Caption)
///   ├── TabPanel (Buy / Sell tabs)
///   ├── ShopItemCard × 3 (buy tab items, 样本数据)
///   ├── InventoryItemRow × 2 (sell tab items, 样本数据)
///   └── Button ("Close", Secondary) — ShopPanelAction::Close
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器
/// - `theme`: 主题 Resource
///
/// # 返回
/// 商店面板容器实体的 Entity
pub fn spawn_shop_panel(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
) -> Entity {
    // ── 1. Container panel ──
    let container = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(container).insert((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(500.0),
            row_gap: Val::Px(theme.spacing.md),
            padding: UiRect::all(Val::Px(theme.spacing.lg)),
            ..default()
        },
        ShopPanel,
        Name::new("ShopPanel"),
    ));

    // ── 2. Header row: Title + Gold display ──
    let header = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            },
            Name::new("ShopHeader"),
        ))
        .id();
    commands.entity(header).set_parent_in_place(container);

    // Title text: "Shop"
    let title = spawn_text(commands, asset_server, theme, "Shop", TextVariant::Heading);
    commands.entity(title).set_parent_in_place(header);

    // 金币显示文本
    let gold_text = spawn_text(
        commands,
        asset_server,
        theme,
        "Gold: 999",
        TextVariant::Caption,
    );
    commands
        .entity(gold_text)
        .insert(TextColor(theme.colors.text_secondary));
    commands.entity(gold_text).set_parent_in_place(header);

    // ── 3. TabPanel (Buy / Sell) ──
    // MVP: uses plain English labels since spawn_tab_panel does not support
    // localization keys yet. Labels are "Buy" and "Sell".
    let tabs = spawn_tab_panel(commands, theme, &["Buy", "Sell"], 0);
    commands.entity(tabs).set_parent_in_place(container);

    // ── 4. Sample buy items (ShopItemCard × 3) ──
    let buy_items: [(&str, u32, u32); 3] = [
        ("Health Potion", 50, 10),
        ("Mana Potion", 80, 5),
        ("Antidote", 30, 3),
    ];

    for (name, price, stock) in buy_items {
        let card = spawn_shop_item_card(commands, asset_server, theme, name, price, stock);
        commands.entity(card).set_parent_in_place(container);
    }

    // ── 5. Sample sell items (InventoryItemRow × 2) ──
    let sell_items: [(&str, u32); 2] = [("Old Sword", 1), ("Leather Armor", 1)];

    for (name, qty) in sell_items {
        let row = spawn_inventory_item_row(commands, asset_server, theme, name, qty);
        commands.entity(row).set_parent_in_place(container);
    }

    // ── 6. Close button (Secondary variant) ──
    let close_btn = spawn_localized_button(
        commands,
        theme,
        loc::ui::CLOSE,
        "Close",
        ButtonVariant::Secondary,
    );
    commands.entity(close_btn).insert(ShopPanelAction::Close);
    commands.entity(close_btn).set_parent_in_place(container);

    container
}
