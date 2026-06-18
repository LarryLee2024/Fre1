//! 制作/锻造 Systems
//!
//! 包括制作、附魔、升级等 Observer。
//! 详见 docs/02-domain/domains/crafting_domain.md §5

use bevy::prelude::*;

use super::super::components::{CraftingStation, EnchantmentSlot, RecipeDef, UpgradeLevel};
use super::super::events::{CraftingFailed, EnchantmentApplied, ItemCrafted, ItemUpgraded};
use super::super::resources::CraftingConfig;
use super::super::rules::{
    check_materials_available, check_station_match, check_upgrade_limit, has_free_enchantment_slot,
    perform_skill_check,
};

/// 处理制作请求。
pub fn on_craft_item(
    _trigger: On<ItemCrafted>,
    config: Res<CraftingConfig>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    // 简化实现：触发制作完成事件
    // 完整实现需检查材料、站台、技能，消耗材料，产生产出
}

/// 处理附魔请求。
pub fn on_apply_enchantment(
    _trigger: On<EnchantmentApplied>,
    mut slot_query: Query<&mut EnchantmentSlot>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if let Ok(mut slot) = slot_query.get_mut(event.entity) {
        if !has_free_enchantment_slot(&slot) {
            commands.trigger(CraftingFailed {
                entity: event.entity,
                recipe_id: event.new_enchantment.clone(),
                fail_reason: "附魔槽位已满".to_string(),
                materials_lost: vec![],
            });
            return;
        }
        slot.active_enchants.push(event.new_enchantment.clone());
    }
}

/// 处理装备升级请求。
pub fn on_upgrade_item(
    _trigger: On<ItemUpgraded>,
    mut upgrade_query: Query<&mut UpgradeLevel>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if let Ok(mut level) = upgrade_query.get_mut(event.entity) {
        if !check_upgrade_limit(&level) {
            commands.trigger(CraftingFailed {
                entity: event.entity,
                recipe_id: format!("upgrade_{}", event.equipment_item),
                fail_reason: "已达最大升级等级".to_string(),
                materials_lost: vec![],
            });
            return;
        }
        let old_level = level.current;
        level.current += 1;
        // ItemUpgraded 已由外部触发
    }
}
