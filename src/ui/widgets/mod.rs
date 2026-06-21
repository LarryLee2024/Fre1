//! 游戏玩法 UI 控件
//!
//! 将原语层组件组合成游戏概念控件。
//! 此层是 Primitives 的唯一使用者；不允许直接操作 Node/Button。
//!
//! 当前 Widget：
//! - SkillSlot — 技能卡控件
//! - ActionMenu — 战斗行动菜单（攻击、防御、技能、物品、等待）
//! - CharacterCard — 角色属性卡（名称、等级、HP/MP 条）
//! - InventoryItemRow — 物品行 Widget（名称、数量、使用按钮）
//! - InventoryGrid — 背包网格有机体（标题、金币、物品列表、关闭按钮）
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
pub mod skill_slot;

use bevy::prelude::*;

use self::action_menu::ActionMenuPlugin;
use self::character_card::CharacterCardPlugin;
use self::inventory_grid::InventoryGridPlugin;
use self::inventory_item_row::InventoryItemRowPlugin;
use self::skill_slot::SkillSlotPlugin;

/// WidgetsPlugin — registers all gameplay UI controls
///
/// Added after PrimitivesPlugin, before ScreenPlugin.
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SkillSlotPlugin,
            ActionMenuPlugin,
            CharacterCardPlugin,
            InventoryGridPlugin,
            InventoryItemRowPlugin,
        ));
    }
}
