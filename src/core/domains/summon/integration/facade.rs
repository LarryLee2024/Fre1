//! SummonReadFacade + SummonWriteFacade — Summon 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Summon 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Summon 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法：立即执行，适合独占 System / 测试
//! - `Commands` 方法：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::summon::components::{SummonAIMode, SummonBond, SummonSlotManager};
use crate::core::domains::summon::resources::SummonConfig;

// ─── SummonReadFacade ─────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Summon 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct SummonReadFacade;

impl SummonReadFacade {
    /// 获取实体的召唤绑定信息。
    ///
    /// # Returns
    /// - `Some(&SummonBond)` — 如果实体拥有 `SummonBond` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_summon_bond(world: &World, entity: Entity) -> Option<&SummonBond> {
        world.get::<SummonBond>(entity)
    }

    /// 获取实体的召唤槽位管理器。
    ///
    /// # Returns
    /// - `Some(&SummonSlotManager)` — 如果实体拥有 `SummonSlotManager` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_slot_manager(world: &World, entity: Entity) -> Option<&SummonSlotManager> {
        world.get::<SummonSlotManager>(entity)
    }

    /// 检查实体是否有空闲召唤槽位。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，默认为无空闲槽位。
    ///
    /// # ReadFacade: 查询空闲槽位
    pub fn has_free_slot(world: &World, entity: Entity) -> bool {
        world
            .get::<SummonSlotManager>(entity)
            .is_some_and(|mgr| mgr.has_free_slot())
    }

    /// 获取实体当前的召唤物数量。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，返回 0。
    ///
    /// # ReadFacade: 查询召唤物数量
    pub fn active_summon_count(world: &World, entity: Entity) -> u32 {
        world
            .get::<SummonSlotManager>(entity)
            .map_or(0, |mgr| mgr.active_summons.len() as u32)
    }

    /// 获取召唤系统全局配置。
    ///
    /// # ReadFacade: 安全查询全局配置
    pub fn get_config(world: &World) -> &SummonConfig {
        world.resource::<SummonConfig>()
    }
}

// ─── SummonWriteFacade ─────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Summon 域组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct SummonWriteFacade;

impl SummonWriteFacade {
    /// 添加召唤物到实体的召唤槽位（立即执行）。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，此操作为空操作。
    ///
    /// # WriteFacade: 立即添加召唤物
    pub fn add_summon(world: &mut World, caster: Entity, summon_entity: Entity) {
        if let Some(mut slot_mgr) = world.get_mut::<SummonSlotManager>(caster) {
            slot_mgr.add_summon(summon_entity);
        }
    }

    /// 从实体的召唤槽位移除召唤物（立即执行）。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，此操作为空操作。
    ///
    /// # WriteFacade: 立即移除召唤物
    pub fn remove_summon(world: &mut World, caster: Entity, summon_entity: Entity) {
        if let Some(mut slot_mgr) = world.get_mut::<SummonSlotManager>(caster) {
            slot_mgr.remove_summon(summon_entity);
        }
    }

    /// 更改召唤物的 AI 模式（立即执行）。
    ///
    /// 如果实体没有 `SummonBond` 组件，此操作为空操作。
    ///
    /// # WriteFacade: 立即修改 AI 模式
    pub fn set_ai_mode(world: &mut World, summon_entity: Entity, mode: SummonAIMode) {
        if let Some(mut bond) = world.get_mut::<SummonBond>(summon_entity) {
            bond.ai_mode = mode;
        }
    }

    /// 初始化实体的召唤槽位管理器（立即执行）。
    ///
    /// 仅在实体尚未拥有 `SummonSlotManager` 时插入。
    ///
    /// # WriteFacade: 立即初始化槽位管理器
    pub fn init_slot_manager(world: &mut World, entity: Entity, max_slots: u32) {
        if world.get::<SummonSlotManager>(entity).is_some() {
            return;
        }
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(SummonSlotManager::new(max_slots));
        }
    }

    /// 添加召唤物到实体的召唤槽位（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟添加召唤物
    pub fn add_summon_deferred(commands: &mut Commands, caster: Entity, summon_entity: Entity) {
        commands
            .entity(caster)
            .insert(SlotManagerAddSummon(summon_entity));
    }

    /// 从实体的召唤槽位移除召唤物（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟移除召唤物
    pub fn remove_summon_deferred(commands: &mut Commands, caster: Entity, summon_entity: Entity) {
        commands
            .entity(caster)
            .insert(SlotManagerRemoveSummon(summon_entity));
    }

    /// 更改召唤物的 AI 模式（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟修改 AI 模式
    pub fn set_ai_mode_deferred(
        commands: &mut Commands,
        summon_entity: Entity,
        mode: SummonAIMode,
    ) {
        commands
            .entity(summon_entity)
            .insert(SummonAIModeOverride(mode));
    }

    /// 初始化实体的召唤槽位管理器（通过 Commands 延迟执行）。
    ///
    /// 通过添加 `InitSlotManager` 标记组件触发初始化。
    ///
    /// # WriteFacade: 延迟初始化槽位管理器
    pub fn init_slot_manager_deferred(commands: &mut Commands, entity: Entity, max_slots: u32) {
        commands.entity(entity).insert(InitSlotManager(max_slots));
    }
}

// ─── Internal marker components for deferred operations ────────────────

/// 标记：延迟添加召唤物操作。
#[derive(Component)]
struct SlotManagerAddSummon(Entity);

/// 标记：延迟移除召唤物操作。
#[derive(Component)]
struct SlotManagerRemoveSummon(Entity);

/// 标记：延迟更改 AI 模式操作。
#[derive(Component)]
struct SummonAIModeOverride(SummonAIMode);

/// 标记：延迟初始化槽位管理器操作。
#[derive(Component)]
struct InitSlotManager(u32);
