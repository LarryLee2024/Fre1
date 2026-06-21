//! ShopItemCard Factory — 商品卡片复合控件的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 UI 节点。
//! 输入 Props + Theme → 输出 Entity。所有子控件通过 Primitives 工厂函数创建。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;

use super::components::{ShopItemAction, ShopItemCard};

/// 工厂函数：生成一个完整的商品卡片控件
///
/// # UI 树结构
///
/// ```text
/// Panel (Card)
///   ├── Text (item name, Caption)
///   ├── Text ("Gold: {price}", Caption, secondary)
///   ├── Text ("Stock: {stock}", Caption, secondary)
///   └── Button ("Buy", Primary) — ShopItemAction::Buy
/// ```
///
/// # 参数
/// - `commands`: ECS 命令
/// - `asset_server`: 资源管理器（传递给文本工厂）
/// - `theme`: 主题 Resource（提供颜色/间距令牌）
/// - `item_name`: 物品显示名称
/// - `price`: 物品价格（金币）
/// - `stock`: 库存数量
///
/// # 返回
/// 商品卡片容器实体的 Entity
pub fn spawn_shop_item_card(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    item_name: impl Into<String>,
    price: u32,
    stock: u32,
) -> Entity {
    let name_str: String = item_name.into();
    let price_str = format!("Gold: {}", price);
    let stock_str = format!("Stock: {}", stock);

    // ── 1. Container panel (Card variant) ──
    let container = spawn_panel(commands, theme, PanelVariant::Card);
    commands.entity(container).insert((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(theme.spacing.xs),
            padding: UiRect::all(Val::Px(theme.spacing.md)),
            ..default()
        },
        ShopItemCard {
            item_id: 0,
            price,
            stock,
        },
        Name::new(format!("ShopItemCard({})", name_str)),
    ));

    // ── 2. Item name text (Caption variant, primary color) ──
    let name_text = spawn_text(
        commands,
        asset_server,
        theme,
        &name_str,
        TextVariant::Caption,
    );
    commands
        .entity(name_text)
        .insert(TextColor(theme.colors.text_primary));
    commands.entity(name_text).set_parent_in_place(container);

    // ── 3. Price text (Caption variant, secondary color) ──
    let price_text = spawn_text(
        commands,
        asset_server,
        theme,
        &price_str,
        TextVariant::Caption,
    );
    commands
        .entity(price_text)
        .insert(TextColor(theme.colors.text_secondary));
    commands.entity(price_text).set_parent_in_place(container);

    // ── 4. Stock text (Caption variant, secondary color) ──
    let stock_text = spawn_text(
        commands,
        asset_server,
        theme,
        &stock_str,
        TextVariant::Caption,
    );
    commands
        .entity(stock_text)
        .insert(TextColor(theme.colors.text_secondary));
    commands.entity(stock_text).set_parent_in_place(container);

    // ── 5. Buy button (Primary variant) ──
    let buy_btn = spawn_localized_button(
        commands,
        theme,
        loc::economy::SHOP_BUY_TEXT,
        "Buy",
        ButtonVariant::Primary,
    );
    commands.entity(buy_btn).insert(ShopItemAction::Buy);
    commands.entity(buy_btn).set_parent_in_place(container);

    container
}
