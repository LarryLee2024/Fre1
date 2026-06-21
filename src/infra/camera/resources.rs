//! Camera Resources — ECS Resource 类型

use bevy::prelude::*;

/// UnitId → WorldPos 解析器（由业务侧注册）。
///
/// Camera 不感知 Domain 类型。此解析器由业务侧（如 Tactical Plugin）在初始化时注册，
/// 实现 UnitId 到世界坐标的转换。
#[derive(Resource, Clone, Copy, Default)]
pub struct UnitPositionResolver(pub Option<fn(u64) -> Vec2>);

/// Tile 尺寸——用于 CameraTarget::TilePos → WorldPos 转换。
///
/// 由场景系统在初始化时设置，匹配当前场景的网格尺寸。
#[derive(Resource, Clone, Copy)]
pub struct TileSize(pub f32);

impl Default for TileSize {
    fn default() -> Self {
        Self(80.0)
    }
}
