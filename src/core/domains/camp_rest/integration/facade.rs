//! CampRestReadFacade + CampRestWriteFacade — CampRest 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 CampRest 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 CampRest 域组件的修改操作。
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::camp_rest::components::{
    CampNPC, CampRestMarker, DiceType, HitDicePool, RestPhase, RestState,
};
use crate::core::domains::camp_rest::failure::CampRestFailure;
use crate::core::domains::camp_rest::systems::CampEventRegistry;

// ─── CampRestReadFacade ─────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 CampRest 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct CampRestReadFacade;

impl CampRestReadFacade {
    /// 获取实体的休息状态。
    ///
    /// # Returns
    /// - `Some(&RestState)` — 如果实体拥有 `RestState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_rest_state(world: &World, entity: Entity) -> Option<&RestState> {
        world.get::<RestState>(entity)
    }

    /// 获取实体的生命骰池。
    ///
    /// # Returns
    /// - `Some(&HitDicePool)` — 如果实体拥有 `HitDicePool` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_hit_dice_pool(world: &World, entity: Entity) -> Option<&HitDicePool> {
        world.get::<HitDicePool>(entity)
    }

    /// 获取实体的营地 NPC 数据。
    ///
    /// # Returns
    /// - `Some(&CampNPC)` — 如果实体拥有 `CampNPC` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_camp_npc(world: &World, entity: Entity) -> Option<&CampNPC> {
        world.get::<CampNPC>(entity)
    }

    /// 检查实体是否拥有 CampRestMarker。
    ///
    /// # ReadFacade: 标记检查
    pub fn has_marker(world: &World, entity: Entity) -> bool {
        world.get::<CampRestMarker>(entity).is_some()
    }

    /// 检查实体是否正在进行休息。
    ///
    /// 返回 `true` 当实体处于 Resting 或 LightActivity 阶段。
    ///
    /// # ReadFacade: 休息状态检查
    pub fn is_resting(world: &World, entity: Entity) -> bool {
        world
            .get::<RestState>(entity)
            .is_some_and(|state| state.phase.is_resting())
    }

    /// 获取实体当前的休息阶段。
    ///
    /// # Returns
    /// - `Some(RestPhase)` — 如果实体拥有 `RestState` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 阶段查询
    pub fn current_phase(world: &World, entity: Entity) -> Option<RestPhase> {
        world.get::<RestState>(entity).map(|state| state.phase)
    }

    /// 获取实体当前可用的生命骰数量。
    ///
    /// # ReadFacade: 生命骰余量查询
    pub fn remaining_hit_dice(world: &World, entity: Entity) -> u32 {
        world
            .get::<HitDicePool>(entity)
            .map_or(0, |pool| pool.current)
    }

    /// 获取实体生命骰池完整信息。
    ///
    /// # Returns
    /// - `Some((current, max, dice_type))` — 如果实体拥有 `HitDicePool` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 生命骰信息查询
    pub fn hit_dice_info(world: &World, entity: Entity) -> Option<(u32, u32, DiceType)> {
        world
            .get::<HitDicePool>(entity)
            .map(|pool| (pool.current, pool.max, pool.dice_type))
    }

    /// 获取营地事件注册表资源。
    ///
    /// # ReadFacade: 资源查询
    pub fn get_camp_event_registry(world: &World) -> Option<&CampEventRegistry> {
        world.get_resource::<CampEventRegistry>()
    }
}

// ─── CampRestWriteFacade ─────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 CampRest 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct CampRestWriteFacade;

impl CampRestWriteFacade {
    /// 开始短休。
    ///
    /// # WriteFacade: 开始短休
    pub fn start_short_rest(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.start_short_rest();
        }
    }

    /// 开始长休。
    ///
    /// # WriteFacade: 开始长休
    pub fn start_long_rest(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.start_long_rest();
        }
    }

    /// 进入长休轻活动阶段。
    ///
    /// # WriteFacade: 轻活动阶段
    pub fn enter_light_activity(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.enter_light_activity();
        }
    }

    /// 标记休息完成。
    ///
    /// # WriteFacade: 完成休息
    pub fn complete_rest(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.complete();
        }
    }

    /// 标记休息失败。
    ///
    /// # WriteFacade: 失败休息
    pub fn fail_rest(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.fail();
        }
    }

    /// 重置休息状态为 Idle。
    ///
    /// # WriteFacade: 重置休息
    pub fn reset_rest_state(world: &mut World, entity: Entity) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.reset();
        }
    }

    /// 消耗生命骰。
    ///
    /// # Errors
    /// - `CampRestFailure::InsufficientHitDice` — 生命骰不足
    ///
    /// # WriteFacade: 安全消耗生命骰
    pub fn spend_hit_dice(
        world: &mut World,
        entity: Entity,
        count: u32,
    ) -> Result<(), CampRestFailure> {
        let mut pool =
            world
                .get_mut::<HitDicePool>(entity)
                .ok_or(CampRestFailure::InsufficientHitDice {
                    available: 0,
                    requested: count,
                })?;

        if pool.spend(count) {
            Ok(())
        } else {
            Err(CampRestFailure::InsufficientHitDice {
                available: pool.current,
                requested: count,
            })
        }
    }

    /// 长休恢复生命骰（不变量 3.4）。
    ///
    /// # WriteFacade: 长休生命骰恢复
    pub fn recover_hit_dice(world: &mut World, entity: Entity) {
        if let Some(mut pool) = world.get_mut::<HitDicePool>(entity) {
            pool.recover_for_long_rest();
        }
    }

    /// 设置生命骰池的最大值（升级时调用）。
    ///
    /// # WriteFacade: 更新生命骰上限
    pub fn set_hit_dice_max(world: &mut World, entity: Entity, new_max: u32) {
        if let Some(mut pool) = world.get_mut::<HitDicePool>(entity) {
            pool.set_max(new_max);
        }
    }

    /// 累加中断时间（分钟）。
    ///
    /// 安全累加中断分钟数，使用 saturating_add 防止溢出。
    ///
    /// # WriteFacade: 更新中断时间
    pub fn add_interrupt_time(world: &mut World, entity: Entity, minutes: u32) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.interrupt_duration = rest.interrupt_duration.saturating_add(minutes);
        }
    }

    /// 设置上次长休完成帧数。
    ///
    /// # WriteFacade: 记录长休时间
    pub fn set_last_long_rest_frame(world: &mut World, entity: Entity, frame: u64) {
        if let Some(mut rest) = world.get_mut::<RestState>(entity) {
            rest.last_long_rest_frame = Some(frame);
        }
    }

    /// 添加对话选项到营地 NPC。
    ///
    /// # WriteFacade: 添加 NPC 对话
    pub fn add_npc_dialogue(world: &mut World, entity: Entity, dialogue: String) {
        if let Some(mut npc) = world.get_mut::<CampNPC>(entity) {
            npc.available_dialogues.push(dialogue);
        }
    }

    /// 生成一个带有营地休息组件的实体。
    ///
    /// 创建的实体包含：
    /// - RestState（Idle）
    /// - HitDicePool
    /// - CampRestMarker
    ///
    /// # WriteFacade: 生成休息实体
    pub fn spawn_rest_entity(
        commands: &mut Commands,
        hit_dice_max: u32,
        dice_type: DiceType,
    ) -> Entity {
        commands
            .spawn((
                RestState::new(),
                HitDicePool::new(hit_dice_max, dice_type),
                CampRestMarker,
            ))
            .id()
    }
}
