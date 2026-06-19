//! 制作/锻造 Systems
//!
//! 包括制作、附魔、升级等 Observer。
//! 详见 docs/02-domain/domains/crafting_domain.md §5

use bevy::prelude::*;

use super::super::components::{EnchantmentSlot, UpgradeLevel};
use super::super::events::{CraftingFailed, EnchantmentApplied, ItemCrafted, ItemUpgraded};
use super::super::resources::{CraftingConfig, EnchantmentDefRegistry};
use super::super::rules::{
    check_enchant_exclusivity, check_upgrade_limit,
    has_free_enchantment_slot,
};

/// 处理制作请求。
pub fn on_craft_item(
    _trigger: On<ItemCrafted>,
    _config: Res<CraftingConfig>,
    _commands: Commands,
) {
    let _event = _trigger.event();
    // 简化实现：触发制作完成事件
    // 完整实现需检查材料、站台、技能，消耗材料，产生产出
}

/// 处理附魔请求。
pub fn on_apply_enchantment(
    _trigger: On<EnchantmentApplied>,
    mut slot_query: Query<&mut EnchantmentSlot>,
    enchant_registry: Res<EnchantmentDefRegistry>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if let Ok(mut slot) = slot_query.get_mut(event.entity) {
        // 1. 检查槽位是否已满
        if !has_free_enchantment_slot(&slot) {
            commands.trigger(CraftingFailed {
                entity: event.entity,
                recipe_id: event.new_enchantment.clone(),
                fail_reason: "附魔槽位已满".to_string(),
                materials_lost: vec![],
            });
            return;
        }

        // 2. 检查附魔定义是否存在
        let Some(new_enchant_def) = enchant_registry.get(&event.new_enchantment) else {
            commands.trigger(CraftingFailed {
                entity: event.entity,
                recipe_id: event.new_enchantment.clone(),
                fail_reason: "附魔定义不存在".to_string(),
                materials_lost: vec![],
            });
            return;
        };

        // 3. 检查互斥规则
        let all_defs: Vec<_> = enchant_registry.defs.values().cloned().collect();
        if let Some(conflict_index) =
            check_enchant_exclusivity(new_enchant_def, &slot.active_enchants, &all_defs)
        {
            let conflict_id = slot.active_enchants[conflict_index].clone();
            commands.trigger(CraftingFailed {
                entity: event.entity,
                recipe_id: event.new_enchantment.clone(),
                fail_reason: format!("与已有附魔 '{}' 互斥", conflict_id),
                materials_lost: vec![],
            });
            return;
        }

        // 3. 应用附魔
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
        let _old_level = level.current;
        level.current += 1;
        // ItemUpgraded 已由外部触发
    }
}
