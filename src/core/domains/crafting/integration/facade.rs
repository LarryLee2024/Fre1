//! CraftingFacade — Crafting 域读写的业务语义 API。
//!
//! 所有 Crafting 域组件（EnchantmentSlot、UpgradeLevel）的字段访问都在此文件中完成。
//! Systems 通过 ReadFacade / WriteFacade 交互，永远不直接访问 Crafting 组件的字段。
//!
//! # 职责边界
//!
//! - 封装对 EnchantmentSlot / UpgradeLevel 的读写
//! - 提供 spawn_enchantable_entity 快捷创建
//! - 🟥 禁止：在此模块之外直接 import EnchantmentSlot / UpgradeLevel 进行字段访问

use bevy::prelude::*;

use crate::core::domains::crafting::components::{EnchantmentDef, EnchantmentSlot, UpgradeLevel};
use crate::core::domains::crafting::resources::{CraftingConfig, EnchantmentDefRegistry};

// ─── 只读操作 ────────────────────────────────────────────────────────────

/// Crafting 域只读查询接口。
///
/// 外部通过此结构体的 static 方法读取 Crafting 域数据。
/// 所有方法接受 `&World`，返回只读引用或拷贝值。
pub struct CraftingReadFacade;

impl CraftingReadFacade {
    // ─── 组件查询 ──────────────────────────────────────────────────────

    /// 获取实体的 EnchantmentSlot 组件引用。
    pub fn enchantment_slot(world: &World, entity: Entity) -> Option<&EnchantmentSlot> {
        world.get::<EnchantmentSlot>(entity)
    }

    /// 获取实体的 UpgradeLevel 组件引用。
    pub fn upgrade_level(world: &World, entity: Entity) -> Option<&UpgradeLevel> {
        world.get::<UpgradeLevel>(entity)
    }

    /// 检查实体是否拥有 EnchantmentSlot 组件。
    pub fn has_enchantment_slot(world: &World, entity: Entity) -> bool {
        world.get::<EnchantmentSlot>(entity).is_some()
    }

    /// 检查实体是否拥有 UpgradeLevel 组件。
    pub fn has_upgrade_level(world: &World, entity: Entity) -> bool {
        world.get::<UpgradeLevel>(entity).is_some()
    }

    // ─── 资源查询 ──────────────────────────────────────────────────────

    /// 获取 CraftingConfig 资源引用。
    pub fn config(world: &World) -> Option<&CraftingConfig> {
        world.get_resource::<CraftingConfig>()
    }

    /// 获取 EnchantmentDefRegistry 资源引用。
    pub fn enchantment_registry(world: &World) -> Option<&EnchantmentDefRegistry> {
        world.get_resource::<EnchantmentDefRegistry>()
    }

    /// 获取指定 ID 的 EnchantmentDef 引用。
    pub fn enchantment_def<'a>(world: &'a World, id: &str) -> Option<&'a EnchantmentDef> {
        world
            .get_resource::<EnchantmentDefRegistry>()
            .and_then(|reg| reg.get(id))
    }

    // ─── 便利查询 ──────────────────────────────────────────────────────

    /// 获取实体当前的升级等级。
    pub fn current_upgrade_level(world: &World, entity: Entity) -> Option<u32> {
        world.get::<UpgradeLevel>(entity).map(|level| level.current)
    }

    /// 获取实体的最大升级等级。
    pub fn max_upgrade_level(world: &World, entity: Entity) -> Option<u32> {
        world.get::<UpgradeLevel>(entity).map(|level| level.max)
    }

    /// 检查实体是否可以继续升级。
    pub fn can_upgrade(world: &World, entity: Entity) -> Option<bool> {
        world
            .get::<UpgradeLevel>(entity)
            .map(|level| level.can_upgrade())
    }

    /// 检查实体是否有空闲的附魔槽位。
    pub fn has_free_enchantment_slot(world: &World, entity: Entity) -> Option<bool> {
        world
            .get::<EnchantmentSlot>(entity)
            .map(|slot| (slot.active_enchants.len() as u32) < slot.max_slots)
    }

    /// 获取实体当前的附魔 ID 列表。
    pub fn active_enchant_ids(world: &World, entity: Entity) -> Option<Vec<String>> {
        world
            .get::<EnchantmentSlot>(entity)
            .map(|slot| slot.active_enchants.clone())
    }

    /// 获取实体的最大附魔槽位数。
    pub fn max_enchantment_slots(world: &World, entity: Entity) -> Option<u32> {
        world
            .get::<EnchantmentSlot>(entity)
            .map(|slot| slot.max_slots)
    }
}

// ─── 写操作 ──────────────────────────────────────────────────────────────

/// Crafting 域写入接口。
///
/// 外部通过此结构体的 static 方法修改 Crafting 域数据。
/// 方法直接修改组件状态（通过 `&mut World` 或 `Commands`），
/// 不触发 Crafting 域的 Observer 事件循环——调用方如需通知其他系统应手动触发事件。
pub struct CraftingWriteFacade;

impl CraftingWriteFacade {
    // ─── 附魔操作 ──────────────────────────────────────────────────────

    /// 为实体添加附魔。
    ///
    /// 直接向 EnchantmentSlot.active_enchants 推入附魔 ID。
    /// 调用方应确保有可用槽位且无互斥冲突。
    /// 不触发 EnchantmentApplied 事件——调用方如需通知应自行触发。
    pub fn apply_enchantment(world: &mut World, entity: Entity, enchantment_id: &str) {
        if let Some(mut slot) = world.get_mut::<EnchantmentSlot>(entity) {
            slot.active_enchants.push(enchantment_id.to_string());
        }
    }

    /// 移除实体指定索引的附魔。
    ///
    /// 索引越界时为 no-op。
    /// 不触发事件——调用方负责在需要时触发。
    pub fn remove_enchantment(world: &mut World, entity: Entity, index: usize) {
        if let Some(mut slot) = world.get_mut::<EnchantmentSlot>(entity)
            && index < slot.active_enchants.len()
        {
            slot.active_enchants.remove(index);
        }
    }

    // ─── 升级操作 ──────────────────────────────────────────────────────

    /// 提升实体的升级等级。
    ///
    /// 直接递增 UpgradeLevel.current。已达最大等级时为 no-op。
    /// 不触发 ItemUpgraded 事件——调用方负责在需要时触发。
    pub fn upgrade_entity(world: &mut World, entity: Entity) {
        if let Some(mut level) = world.get_mut::<UpgradeLevel>(entity)
            && level.can_upgrade()
        {
            level.current += 1;
        }
    }

    /// 设置实体的升级等级为指定值。
    ///
    /// 不会超过最大值。不触发 ItemUpgraded 事件。
    /// 主要用于恢复存档或初始化。
    pub fn set_upgrade_level(world: &mut World, entity: Entity, level: u32) {
        if let Some(mut upgrade) = world.get_mut::<UpgradeLevel>(entity) {
            upgrade.current = level.min(upgrade.max);
        }
    }

    // ─── 实体创建 ──────────────────────────────────────────────────────

    /// 生成一个可附魔/可升级的实体。
    ///
    /// 创建的实体包含：
    /// - EnchantmentSlot（指定槽位数的空附魔槽）
    /// - UpgradeLevel（指定最大等级的 0 级）
    pub fn spawn_enchantable_entity(
        commands: &mut Commands,
        max_slots: u32,
        max_upgrade: u32,
    ) -> Entity {
        commands
            .spawn((
                EnchantmentSlot {
                    max_slots,
                    active_enchants: Vec::new(),
                },
                UpgradeLevel::new(max_upgrade),
            ))
            .id()
    }
}
