//! TerrainQueryParam — Bevy SystemParam，封装所有 Terrain 域组件查询。
//!
//! Systems 通过此 param 读取地形数据，完全不知道 `TilePos` /
//! `TileProperties` / `SurfaceOverride` 等组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     terrain_query: TerrainQueryParam,
//!     // ...
//! ) {
//!     if let Some(tile_props) = terrain_query.get_tile_properties(entity) {
//!         // 读取地形属性
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `TerrainWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::terrain::components::{
    HazardTriggeredState, SurfaceOverride, TerrainAttachEffect, TilePos, TileProperties,
};

/// 地形查询 SystemParam — 封装所有 Terrain 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&TilePos>` + `Query<&TileProperties>`。
#[derive(SystemParam)]
pub struct TerrainQueryParam<'w, 's> {
    /// 地形网格坐标只读查询
    tile_pos_query: Query<'w, 's, &'static TilePos>,
    /// 地形属性只读查询
    tile_properties_query: Query<'w, 's, &'static TileProperties>,
    /// 表面覆盖只读查询
    surface_override_query: Query<'w, 's, &'static SurfaceOverride>,
    /// 地形效果记录只读查询
    terrain_attach_effect_query: Query<'w, 's, &'static TerrainAttachEffect>,
    /// 陷阱触发状态只读查询
    hazard_triggered_state_query: Query<'w, 's, &'static HazardTriggeredState>,
}

impl<'w, 's> TerrainQueryParam<'w, 's> {
    /// 获取实体的地形网格坐标。
    ///
    /// # Returns
    /// - `Some(&TilePos)` — 如果实体拥有 `TilePos` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_tile_pos(&self, entity: Entity) -> Option<&TilePos> {
        self.tile_pos_query.get(entity).ok()
    }

    /// 获取实体的地形属性集合。
    ///
    /// # Returns
    /// - `Some(&TileProperties)` — 如果实体拥有 `TileProperties` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_tile_properties(&self, entity: Entity) -> Option<&TileProperties> {
        self.tile_properties_query.get(entity).ok()
    }

    /// 获取格子的表面覆盖记录。
    ///
    /// # Returns
    /// - `Some(&SurfaceOverride)` — 如果格子上存在表面覆盖
    /// - `None` — 无表面覆盖
    pub fn get_surface_override(&self, entity: Entity) -> Option<&SurfaceOverride> {
        self.surface_override_query.get(entity).ok()
    }

    /// 获取绑定的地形效果记录。
    ///
    /// # Returns
    /// - `Some(&TerrainAttachEffect)` — 如果格子绑定了地形效果
    /// - `None` — 无绑定效果
    pub fn get_terrain_attach_effect(&self, entity: Entity) -> Option<&TerrainAttachEffect> {
        self.terrain_attach_effect_query.get(entity).ok()
    }

    /// 检查实体是否拥有消耗型陷阱触发记录。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `HazardTriggeredState` 组件
    /// - `false` — 无该组件
    pub fn has_hazard_triggered_state(&self, entity: Entity) -> bool {
        self.hazard_triggered_state_query.get(entity).is_ok()
    }

    /// 获取实体的消耗型陷阱触发状态。
    ///
    /// # Returns
    /// - `Some(&HazardTriggeredState)` — 如果实体拥有此组件
    /// - `None` — 无触发记录
    pub fn get_hazard_triggered_state(&self, entity: Entity) -> Option<&HazardTriggeredState> {
        self.hazard_triggered_state_query.get(entity).ok()
    }
}
