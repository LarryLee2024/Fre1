//! SpellReadFacade + SpellWriteFacade — Spell 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Spell 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Spell 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法：立即执行，适合独占 System / 测试
//! - `Commands` 方法：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::spell::components::{
    Concentration, SpellConfig, SpellDefId, SpellLevel, SpellSlotPool, Spellbook,
};
use crate::core::domains::spell::failure::SpellFailure;

// ─── SpellReadFacade ─────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Spell 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct SpellReadFacade;

impl SpellReadFacade {
    /// 获取实体的法术位池。
    ///
    /// # Returns
    /// - `Some(&SpellSlotPool)` — 如果实体拥有 `SpellSlotPool` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询法术位池
    pub fn get_spell_slots(world: &World, entity: Entity) -> Option<&SpellSlotPool> {
        world.get::<SpellSlotPool>(entity)
    }

    /// 获取实体的法术书。
    ///
    /// # Returns
    /// - `Some(&Spellbook)` — 如果实体拥有 `Spellbook` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询法术书
    pub fn get_spellbook(world: &World, entity: Entity) -> Option<&Spellbook> {
        world.get::<Spellbook>(entity)
    }

    /// 检查实体是否正在进行专注施法。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `Concentration` 组件
    /// - `false` — 实体无专注状态
    ///
    /// # ReadFacade: 检查专注状态是否存在
    pub fn has_concentration(world: &World, entity: Entity) -> bool {
        world.get::<Concentration>(entity).is_some()
    }

    /// 获取实体的专注状态详情。
    ///
    /// # Returns
    /// - `Some(&Concentration)` — 如果实体正在专注
    /// - `None` — 无专注状态
    ///
    /// # ReadFacade: 安全查询专注详情
    pub fn get_concentration(world: &World, entity: Entity) -> Option<&Concentration> {
        world.get::<Concentration>(entity)
    }

    /// 查询实体在指定环阶的剩余法术位数。
    ///
    /// # ReadFacade: 查询法术位余量
    pub fn remaining_slots(world: &World, entity: Entity, level: SpellLevel) -> u32 {
        world
            .get::<SpellSlotPool>(entity)
            .map_or(0, |pool| pool.remaining(level))
    }

    /// 获取法术系统全局配置。
    ///
    /// # ReadFacade: 安全查询全局配置
    pub fn get_spell_config(world: &World) -> &SpellConfig {
        world.resource::<SpellConfig>()
    }
}

// ─── SpellWriteFacade ─────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Spell 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct SpellWriteFacade;

impl SpellWriteFacade {
    /// 消耗一个指定环阶的法术位。
    ///
    /// 这是 `SpellSlotPool::consume()` 的 facade 包装。
    /// 调用前应通过 `check_slot_available` 规则校验。
    ///
    /// # Errors
    /// - `SpellFailure::InsufficientSlots` — 法术位不足或实体无 `SpellSlotPool` 组件
    ///
    /// # WriteFacade: 安全消耗法术位
    pub fn consume_slot(
        world: &mut World,
        entity: Entity,
        level: SpellLevel,
        spell_id: &SpellDefId,
    ) -> Result<(), SpellFailure> {
        let mut slot_pool = world.get_mut::<SpellSlotPool>(entity).ok_or_else(|| {
            SpellFailure::InsufficientSlots {
                spell_id: spell_id.clone(),
                required_level: level.as_u8(),
            }
        })?;

        if slot_pool.consume(level) {
            Ok(())
        } else {
            Err(SpellFailure::InsufficientSlots {
                spell_id: spell_id.clone(),
                required_level: level.as_u8(),
            })
        }
    }

    /// 恢复指定环阶的一个法术位。
    ///
    /// # WriteFacade: 安全恢复单法术位
    pub fn restore_slot(world: &mut World, entity: Entity, level: SpellLevel) {
        if let Some(mut slot_pool) = world.get_mut::<SpellSlotPool>(entity) {
            slot_pool.restore_one(level);
        }
    }

    /// 恢复实体的所有法术位（长休恢复）。
    ///
    /// # WriteFacade: 安全恢复全部法术位
    pub fn restore_all_slots(world: &mut World, entity: Entity) {
        if let Some(mut slot_pool) = world.get_mut::<SpellSlotPool>(entity) {
            slot_pool.restore_all();
        }
    }

    /// 添加已知法术到实体的法术书。
    ///
    /// # WriteFacade: 安全添加已知法术
    pub fn add_known_spell(world: &mut World, entity: Entity, spell_id: &SpellDefId) {
        if let Some(mut spellbook) = world.get_mut::<Spellbook>(entity)
            && !spellbook.known_spells.contains(spell_id)
        {
            spellbook.known_spells.push(spell_id.clone());
        }
    }

    /// 添加准备法术到实体的法术书。
    ///
    /// # WriteFacade: 安全添加准备法术
    pub fn add_prepared_spell(world: &mut World, entity: Entity, spell_id: &SpellDefId) {
        if let Some(mut spellbook) = world.get_mut::<Spellbook>(entity)
            && !spellbook.prepared_spells.contains(spell_id)
        {
            spellbook.prepared_spells.push(spell_id.clone());
        }
    }

    /// 从实体的法术书中移除准备法术。
    ///
    /// # WriteFacade: 安全移除准备法术
    pub fn remove_prepared_spell(world: &mut World, entity: Entity, spell_id: &SpellDefId) {
        if let Some(mut spellbook) = world.get_mut::<Spellbook>(entity) {
            spellbook.prepared_spells.retain(|s| s != spell_id);
        }
    }

    /// 设置专注状态（通过 Commands 延迟执行）。
    ///
    /// 添加 `Concentration` 组件到施法者实体。
    /// 如果实体已存在专注状态，旧的会被覆盖。
    ///
    /// # WriteFacade: 通过 Commands 设置专注
    pub fn set_concentration(
        commands: &mut Commands,
        caster: Entity,
        spell_id: SpellDefId,
        total_duration: u32,
        con_modifier: i32,
    ) {
        commands
            .entity(caster)
            .insert(Concentration::new(spell_id, total_duration, con_modifier));
    }

    /// 移除专注状态（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除专注
    pub fn remove_concentration(commands: &mut Commands, caster: Entity) {
        commands.entity(caster).remove::<Concentration>();
    }

    /// 设置专注状态（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即设置专注
    pub fn set_concentration_immediate(
        world: &mut World,
        caster: Entity,
        spell_id: SpellDefId,
        total_duration: u32,
        con_modifier: i32,
    ) {
        if let Ok(mut entity) = world.get_entity_mut(caster) {
            entity.insert(Concentration::new(spell_id, total_duration, con_modifier));
        }
    }

    /// 移除专注状态（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即移除专注
    pub fn remove_concentration_immediate(world: &mut World, caster: Entity) {
        if let Ok(mut entity) = world.get_entity_mut(caster) {
            entity.remove::<Concentration>();
        }
    }
}
