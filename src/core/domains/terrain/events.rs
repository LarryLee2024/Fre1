//! 领域事件 — Terrain 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/terrain_domain.md §6

use crate::shared::ids::DefinitionId;
use bevy::prelude::*;

use super::components::{SurfaceType, TilePos};

/// 单位进入格子时触发。
///
/// 订阅者：
/// - Effect：施加格子上的 TerrainEffect（如进入毒池→施加中毒）
/// - Resource：触发格子上 HazardZone 的检测
/// - Trigger：触发"进入地形"相关触发器
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TileEntered {
    /// 进入格子的单位
    pub entity: Entity,
    /// 目标格子位置
    pub tile: TilePos,
    /// 格子当前表面类型
    pub surface: SurfaceType,
}

/// 格子表面类型变化时触发。
///
/// 订阅者：
/// - Terrain：更新显示、启动恢复计时器
/// - Cue：播放表面变化特效（如冻结/燃烧动画）
/// - UI：更新格子视觉效果
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SurfaceChanged {
    /// 发生变化的格子位置
    pub tile: TilePos,
    /// 变化前的表面类型
    pub old_surface: SurfaceType,
    /// 变化后的表面类型
    pub new_surface: SurfaceType,
}

/// 陷阱触发时触发。
///
/// 订阅者：
/// - Combat：处理伤害
/// - Effect：施加陷阱附带的效果
/// - Cue：陷阱触发特效
#[derive(Event, Debug, Clone, PartialEq)]
pub struct HazardTriggered {
    /// 陷阱所在格子位置
    pub tile: TilePos,
    /// 触发陷阱的目标单位
    pub target: Entity,
    /// 触发的陷阱 ID
    pub hazard_id: String,
}

/// 地形效果施加时触发。
///
/// 订阅者：
/// - UI：显示地形状态图标
/// - Cue：地形效果视觉反馈
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TerrainEffectApplied {
    /// 被施加效果的单位
    pub entity: Entity,
    /// 效果所在格子位置
    pub tile: TilePos,
    /// 施加的效果 ID
    pub effect_id: DefinitionId,
}
