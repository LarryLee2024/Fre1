//! CameraTarget — 镜头目标值对象
//!
//! 使用领域 ID（UnitId/TilePos/Vec2）而非 ECS Entity 的原因：
//! - Entity 是 ECS 运行时概念，可能在聚焦期间被销毁/回收
//! - UnitId 是领域身份，不受 Entity 生命周期影响
//! - 符合 Definition/Instance 分离原则

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 镜头目标——目标位置的抽象表示。
///
/// 不自立为 ECS Component 或 Resource，而是作为值对象嵌入
/// CameraRequest、CameraState、CameraCommand 中。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub enum CameraTarget {
    /// 绝对世界坐标位置。准确度：精确世界坐标，不受网格对齐影响。
    WorldPos(Vec2),

    /// 网格坐标位置 (行, 列)。系统内部将 TilePos 转换为世界坐标。
    TilePos(i32, i32),

    /// 单位 ID。系统内部通过注册的解析器查询单位当前位置。
    UnitId(u64),
}

impl CameraTarget {
    /// 解析 CameraTarget 为世界坐标 Vec2。
    ///
    /// 参数 `unit_positions` 是一个函数引用，由业务侧注册，
    /// 用于将 UnitId 解析为世界坐标。返回 None 表示目标不存在。
    ///
    /// 参数 `tile_size` 用于 TilePos → WorldPos 的转换。
    pub fn resolve(&self, unit_positions: &impl Fn(u64) -> Option<Vec2>, tile_size: f32) -> Vec2 {
        match self {
            CameraTarget::WorldPos(pos) => *pos,
            CameraTarget::TilePos(x, y) => Vec2::new(*x as f32 * tile_size, *y as f32 * tile_size),
            CameraTarget::UnitId(id) => unit_positions(*id).unwrap_or_default(),
        }
    }
}
