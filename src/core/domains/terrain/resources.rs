//! Resources — 地形领域全局资源
//!
//! 包含陷阱定义注册表、地形交互定义等静态配置数据。
//!
//! 详见 docs/04-data/domains/terrain_schema.md §1.4

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::domains::terrain::components::{SurfaceType, TilePos, TileProperties};

/// 陷阱/危险区域定义（Definition 层 — 静态配置，运行时只读）。
#[derive(Debug, Clone)]
pub struct HazardZoneDef {
    /// 陷阱唯一标识
    pub id: String,
    /// 触发条件
    pub trigger_condition: HazardTriggerCondition,
    /// 触发后执行的效果 ID 列表
    pub effects: Vec<String>,
    /// 是否为消耗型（一次触发后失效）
    pub is_consumable: bool,
    /// 可见性控制
    pub visibility: HazardVisibility,
}

impl HazardZoneDef {
    /// 检查该陷阱定义是否匹配给定格子的属性。
    ///
    /// 当前简化实现：匹配特定表面类型或标签。
    /// 将来扩展为使用 AreaDefinition 进行位置匹配。
    /// 检查该陷阱定义是否匹配给定格子的属性。
    ///
    /// TODO[P2][Terrain]: 实现 AreaDefinition 区域匹配逻辑
    ///   当前返回 false（安全默认），待 AreaDefinition 定型后补充完整实现。
    pub fn matches_tile(&self, _props: &TileProperties) -> bool {
        false
    }
}

/// 陷阱触发条件。
#[derive(Debug, Clone)]
pub enum HazardTriggerCondition {
    /// 单位进入时触发
    OnEnter,
    /// 单位在区域内停留时每回合触发
    OnStay,
    /// 回合结束时触发
    OnRoundEnd,
    /// 被技能/效果主动激活
    OnActivate,
}

/// 陷阱可见性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HazardVisibility {
    /// 开战时可见
    Visible,
    /// 隐藏，需要侦查发现
    Hidden,
    /// 特定条件触发后可见
    RevealedOn,
}

/// 陷阱定义注册表（Resource）。
///
/// 存储所有已注册的 HazardZoneDef，供 hazard_system 查询。
/// 由内容加载系统在游戏启动时填充。
#[derive(Resource, Debug, Clone)]
pub struct HazardZoneRegistry {
    /// 所有陷阱定义
    pub zones: Vec<HazardZoneDef>,
}

impl HazardZoneRegistry {
    /// 创建空的陷阱注册表。
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    /// 注册一个陷阱定义。
    pub fn register(&mut self, zone: HazardZoneDef) {
        self.zones.push(zone);
    }
}

impl Default for HazardZoneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TileEntityMap — 空间索引
// ============================================================================

/// TilePos → Entity 映射（空间索引）。
///
/// 提供 O(1) 的 TilePos → Entity 查询，避免在事件处理器中线性扫描所有 TilePos。
/// 由 `update` system 在 PostUpdate 中维护。
#[derive(Resource, Default, Debug, Clone)]
pub struct TileEntityMap {
    map: HashMap<TilePos, Entity>,
}

impl TileEntityMap {
    /// 根据 TilePos 查找对应的 Entity。
    pub fn get(&self, pos: &TilePos) -> Option<Entity> {
        self.map.get(pos).copied()
    }

    /// 更新映射表（在 PostUpdate 中运行）。
    pub fn update(mut map: ResMut<Self>, tile_query: Query<(Entity, &TilePos)>) {
        map.map.clear();
        for (entity, pos) in tile_query.iter() {
            map.map.insert(*pos, entity);
        }
    }
}

/// 地形交互定义（Definition 层 — 静态配置）。
#[derive(Debug, Clone)]
pub struct TerrainInteractionDef {
    /// 目标表面类型
    pub target_surface: SurfaceType,
    /// 持续时间
    pub duration: InteractionDuration,
    /// 是否与现有表面冲突检查
    pub conflict_check: bool,
}

/// 交互持续时间。
#[derive(Debug, Clone)]
pub enum InteractionDuration {
    /// 立即生效，不可恢复
    Instant,
    /// 持续 N 回合后恢复
    Timed { turns: u32 },
    /// 永久改变（需 [Data Exemption]）
    Permanent,
}
