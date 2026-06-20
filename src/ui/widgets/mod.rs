//! Module Name: Widgets — 游戏业务控件
//!
//! 组合 Primitives 层基础组件为游戏概念控件。
//! 本层是 Primitives 的唯一消费者，禁止直接操作 Node/Button 等 Bevy 原语。
//!
//! 当前包含：
//! - SkillSlot — 技能槽卡片控件
//!
//! 后续添加：
//! - CharacterCard
//! - BuffIcon
//! - InventoryGrid
//! - BattleLog
//! - ActionMenu
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod skill_slot;

use bevy::prelude::*;

use self::skill_slot::SkillSlotPlugin;

/// WidgetsPlugin — 注册所有游戏业务控件
///
/// 在 PrimitivesPlugin 之后、Screen Plugin 之前注册。
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SkillSlotPlugin);
    }
}
