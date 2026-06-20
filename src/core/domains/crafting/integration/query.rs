//! CraftingQueryParam — Bevy SystemParam，封装所有 Crafting 组件查询。
//!
//! Systems 通过此 param 获取 Crafting 域数据，完全不知道
//! EnchantmentSlot / UpgradeLevel 组件内部细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     crafting_query: CraftingQueryParam,
//!     // ...
//! ) {
//!     if let Some(slot) = crafting_query.enchantment_slot(entity) {
//!         println!("Slots: {}/{}", slot.active_enchants.len(), slot.max_slots);
//!     }
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::crafting::components::{EnchantmentSlot, UpgradeLevel};
use crate::core::domains::crafting::resources::{CraftingConfig, EnchantmentDefRegistry};

/// Crafting 查询参数 — 封装所有 Crafting 组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&EnchantmentSlot>` + `Res<CraftingConfig>` 等。
/// 函数体内所有 Crafting 数据访问都通过此 param 的字段完成。
#[derive(SystemParam)]
pub struct CraftingQueryParam<'w, 's> {
    /// EnchantmentSlot 组件查询。
    pub enchantment_slots: Query<'w, 's, &'static EnchantmentSlot>,
    /// UpgradeLevel 组件查询。
    pub upgrade_levels: Query<'w, 's, &'static UpgradeLevel>,
    /// 制作系统配置资源。
    pub crafting_config: Res<'w, CraftingConfig>,
    /// 附魔定义注册表资源。
    pub enchantment_registry: Res<'w, EnchantmentDefRegistry>,
}

impl<'w, 's> CraftingQueryParam<'w, 's> {
    /// 获取实体当前的升级等级。
    pub fn current_upgrade_level(&self, entity: Entity) -> Option<u32> {
        self.upgrade_levels
            .get(entity)
            .ok()
            .map(|level| level.current)
    }

    /// 获取实体的最大升级等级。
    pub fn max_upgrade_level(&self, entity: Entity) -> Option<u32> {
        self.upgrade_levels.get(entity).ok().map(|level| level.max)
    }

    /// 检查实体是否可以继续升级。
    pub fn can_upgrade(&self, entity: Entity) -> Option<bool> {
        self.upgrade_levels
            .get(entity)
            .ok()
            .map(|level| level.can_upgrade())
    }

    /// 检查实体是否有空闲的附魔槽位。
    pub fn has_free_enchantment_slot(&self, entity: Entity) -> Option<bool> {
        self.enchantment_slots
            .get(entity)
            .ok()
            .map(|slot| (slot.active_enchants.len() as u32) < slot.max_slots)
    }

    /// 获取实体当前的附魔 ID 列表。
    pub fn active_enchant_ids(&self, entity: Entity) -> Option<Vec<String>> {
        self.enchantment_slots
            .get(entity)
            .ok()
            .map(|slot| slot.active_enchants.clone())
    }

    /// 获取实体的最大附魔槽位数。
    pub fn max_enchantment_slots(&self, entity: Entity) -> Option<u32> {
        self.enchantment_slots
            .get(entity)
            .ok()
            .map(|slot| slot.max_slots)
    }
}
