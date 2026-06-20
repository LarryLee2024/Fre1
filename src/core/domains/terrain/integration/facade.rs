//! TerrainReadFacade + TerrainWriteFacade — Terrain 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Terrain 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Terrain 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法：立即执行，适合独占 System / 测试
//! - `Commands` 方法：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::terrain::components::{
    Concealment, HazardTriggeredState, Passability, SurfaceOverride, TerrainAttachEffect, TilePos,
    TileProperties,
};
use crate::core::domains::terrain::resources::{HazardZoneRegistry, TileEntityMap};

// ─── TerrainReadFacade ─────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Terrain 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct TerrainReadFacade;

impl TerrainReadFacade {
    /// 获取实体的地形网格坐标。
    ///
    /// # Returns
    /// - `Some(&TilePos)` — 如果实体拥有 `TilePos` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询坐标
    pub fn get_tile_pos(world: &World, entity: Entity) -> Option<&TilePos> {
        world.get::<TilePos>(entity)
    }

    /// 获取实体的地形属性集合。
    ///
    /// # Returns
    /// - `Some(&TileProperties)` — 如果实体拥有 `TileProperties` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询地形属性
    pub fn get_tile_properties(world: &World, entity: Entity) -> Option<&TileProperties> {
        world.get::<TileProperties>(entity)
    }

    /// 获取格子的表面覆盖记录。
    ///
    /// # Returns
    /// - `Some(&SurfaceOverride)` — 如果格子上存在表面覆盖
    /// - `None` — 无表面覆盖
    ///
    /// # ReadFacade: 安全查询表面覆盖
    pub fn get_surface_override(world: &World, entity: Entity) -> Option<&SurfaceOverride> {
        world.get::<SurfaceOverride>(entity)
    }

    /// 获取绑定的地形效果记录。
    ///
    /// # Returns
    /// - `Some(&TerrainAttachEffect)` — 如果格子绑定了地形效果
    /// - `None` — 无绑定效果
    ///
    /// # ReadFacade: 安全查询地形效果
    pub fn get_terrain_attach_effect(
        world: &World,
        entity: Entity,
    ) -> Option<&TerrainAttachEffect> {
        world.get::<TerrainAttachEffect>(entity)
    }

    /// 检查实体是否拥有消耗型陷阱触发记录。
    ///
    /// # ReadFacade: 检查陷阱触发记录是否存在
    pub fn has_hazard_triggered_state(world: &World, entity: Entity) -> bool {
        world.get::<HazardTriggeredState>(entity).is_some()
    }

    /// 获取实体的消耗型陷阱触发状态。
    ///
    /// # Returns
    /// - `Some(&HazardTriggeredState)` — 如果实体拥有此组件
    /// - `None` — 无触发记录
    ///
    /// # ReadFacade: 安全查询陷阱触发状态
    pub fn get_hazard_triggered_state(
        world: &World,
        entity: Entity,
    ) -> Option<&HazardTriggeredState> {
        world.get::<HazardTriggeredState>(entity)
    }

    /// 获取地形空间索引资源。
    ///
    /// # ReadFacade: 安全查询资源
    pub fn get_tile_entity_map(world: &World) -> Option<&TileEntityMap> {
        world.get_resource::<TileEntityMap>()
    }

    /// 获取陷阱定义注册表资源。
    ///
    /// # ReadFacade: 安全查询资源
    pub fn get_hazard_zone_registry(world: &World) -> Option<&HazardZoneRegistry> {
        world.get_resource::<HazardZoneRegistry>()
    }

    /// 根据 TilePos 查找对应的格子 Entity。
    ///
    /// 委托给 `TileEntityMap::get()`。
    ///
    /// # Returns
    /// - `Some(Entity)` — 如果该坐标有对应的实体
    /// - `None` — 无对应实体
    ///
    /// # ReadFacade: 安全空间索引查询
    pub fn get_entity_at_tile(world: &World, pos: &TilePos) -> Option<Entity> {
        world
            .get_resource::<TileEntityMap>()
            .and_then(|map| map.get(pos))
    }

    /// 获取格子的当前通行性。
    ///
    /// 委托给 `TileProperties::current_passability()`。
    ///
    /// # Returns
    /// - `Some(Passability)` — 如果实体拥有 `TileProperties` 组件
    /// - `None` — 无该组件
    ///
    /// # ReadFacade: 安全通行性查询
    pub fn current_passability(world: &World, entity: Entity) -> Option<Passability> {
        world
            .get::<TileProperties>(entity)
            .map(|props| props.current_passability())
    }

    /// 获取格子的当前遮蔽度。
    ///
    /// 委托给 `TileProperties::current_concealment()`。
    ///
    /// # Returns
    /// - `Some(Concealment)` — 如果实体拥有 `TileProperties` 组件
    /// - `None` — 无该组件
    ///
    /// # ReadFacade: 安全遮蔽度查询
    pub fn current_concealment(world: &World, entity: Entity) -> Option<Concealment> {
        world
            .get::<TileProperties>(entity)
            .map(|props| props.current_concealment())
    }
}

// ─── TerrainWriteFacade ─────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Terrain 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct TerrainWriteFacade;

impl TerrainWriteFacade {
    /// 设置格子的地形属性（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即设置地形属性
    pub fn set_tile_properties(world: &mut World, entity: Entity, properties: TileProperties) {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            entity.insert(properties);
        }
    }

    /// 设置格子的表面覆盖（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即设置表面覆盖
    pub fn set_surface_override(
        world: &mut World,
        entity: Entity,
        surface_override: SurfaceOverride,
    ) {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            entity.insert(surface_override);
        }
    }

    /// 移除格子的表面覆盖并恢复原始表面（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即移除表面覆盖
    pub fn remove_surface_override(world: &mut World, entity: Entity) {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            entity.remove::<SurfaceOverride>();
        }
    }

    /// 标记陷阱为已消耗（立即执行，通过 &mut World）。
    ///
    /// # WriteFacade: 立即标记陷阱消耗
    pub fn consume_hazard(world: &mut World, entity: Entity, hazard_id: String) {
        if let Some(mut state) = world.get_mut::<HazardTriggeredState>(entity) {
            state.consume(hazard_id);
        }
    }

    /// 设置格子的地形属性（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟设置地形属性
    pub fn set_tile_properties_commands(
        commands: &mut Commands,
        entity: Entity,
        properties: TileProperties,
    ) {
        commands.entity(entity).insert(properties);
    }

    /// 设置格子的表面覆盖（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟设置表面覆盖
    pub fn set_surface_override_commands(
        commands: &mut Commands,
        entity: Entity,
        surface_override: SurfaceOverride,
    ) {
        commands.entity(entity).insert(surface_override);
    }

    /// 移除格子的表面覆盖（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 延迟移除表面覆盖
    pub fn remove_surface_override_commands(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).remove::<SurfaceOverride>();
    }
}
