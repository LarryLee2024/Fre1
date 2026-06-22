//! 模块名：Widgets — 游戏 UI 控件
//!
//! 将 Primitives 层组件组合成游戏概念控件。
//! 本层是 Primitives 的唯一消费者；不允许直接操作 Node/Button。
//!
//! 当前 Widget：
//! - SkillSlot — 技能卡片控件
//! - ActionMenu — 战斗行动菜单（攻击、防御、技能、道具、等待）
//! - CharacterCard — 角色属性卡片（名称、等级、HP/MP 条）
//! - InventoryItemRow — 物品行 Widget（名称、数量、使用按钮）
//! - InventoryGrid — 背包网格有机体（标题、金币、物品列表、关闭按钮）
//! - ShopItemCard — 商店物品卡片 Widget（名称、价格、库存、购买按钮）
//! - ShopPanel — 商店面板有机体（标题栏、标签面板、物品列表、关闭按钮）
//! - TurnOrderBar — 战斗底部行动顺序栏（MVP 静态占位数据）
//!
//! 未来 Widget：
//! - BuffIcon
//! - BattleLog
//!
//! 参见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod action_menu;
pub mod character_card;
pub mod inventory_grid;
pub mod inventory_item_row;
pub mod shop_item_card;
pub mod shop_panel;
pub mod skill_panel;
pub mod skill_slot;
pub mod turn_order_bar;

use bevy::prelude::*;

use self::action_menu::ActionMenuPlugin;
use self::character_card::CharacterCardPlugin;
use self::inventory_grid::InventoryGridPlugin;
use self::inventory_item_row::InventoryItemRowPlugin;
use self::shop_item_card::ShopItemCardPlugin;
use self::shop_panel::ShopPanelPlugin;
use self::skill_panel::SkillPanelPlugin;
use self::skill_slot::SkillSlotPlugin;
use self::turn_order_bar::TurnOrderBarPlugin;

/// WidgetsPlugin — 注册所有游戏 UI 控件
///
/// 在 PrimitivesPlugin 之后、ScreenPlugin 之前添加。
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SkillPanelPlugin,
            SkillSlotPlugin,
            ActionMenuPlugin,
            CharacterCardPlugin,
            InventoryGridPlugin,
            InventoryItemRowPlugin,
            ShopItemCardPlugin,
            ShopPanelPlugin,
            TurnOrderBarPlugin,
        ));
    }
}
