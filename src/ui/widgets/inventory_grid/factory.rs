//! InventoryGrid Factory — 背包物品网格的创建入口

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::list::{components::ListVariant, factory::spawn_list};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_item_row::factory::spawn_inventory_item_row;

use super::components::{InventoryGrid, InventoryGridAction};

/// 生成完整背包物品网格
pub fn spawn_inventory_grid(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
) -> Entity {
    let root = spawn_panel(commands, theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(theme.spacing.md)),
            ..default()
        },
        InventoryGrid,
        Name::new("InventoryGrid"),
    ));

    // 标题
    let title = spawn_text(commands, asset_server, theme, "Inventory", TextVariant::Heading);
    commands.entity(title).set_parent_in_place(root);

    // 金币显示
    let gold = spawn_text(commands, asset_server, theme, "Gold: 100", TextVariant::Caption);
    commands.entity(gold).set_parent_in_place(root);

    // 物品列表
    let list = spawn_list(commands, theme, ListVariant::Vertical);
    commands.entity(list).set_parent_in_place(root);

    // 示例物品行
    for (name, qty) in [("Health Potion", 3), ("Mana Crystal", 1), ("Iron Sword", 1)] {
        let row = spawn_inventory_item_row(commands, asset_server, theme, name, qty);
        commands.entity(row).set_parent_in_place(list);
    }

    // 关闭按钮
    let close = spawn_localized_button(commands, theme, loc::ui::CLOSE, "Close", ButtonVariant::Secondary);
    commands.entity(close).insert(InventoryGridAction::Close);
    commands.entity(close).set_parent_in_place(root);

    root
}
