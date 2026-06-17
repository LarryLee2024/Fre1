//! 地形移动消耗计算 — 纯函数
//!
//! 根据地形类型和移动类型计算单位进入一个格子的实际消耗倍率。
//! 战术域的移动系统通过调用此函数获取地形消耗因子。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.1

use crate::core::domains::terrain::components::TerrainType;

/// 移动类别 — Terrain 域对移动方式的分类。
///
/// 与 Tactical 域的 MovementType 概念对齐，保持域间独立。
/// 当从 Tactical 集成时，通过简单映射将 MovementType 转换为 MoveCategory。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveCategory {
    /// 步行
    Walk,
    /// 飞行
    Fly,
    /// 游泳
    Swim,
    /// 攀爬
    Climb,
    /// 瞬移
    Teleport,
}

/// 计算单位进入指定地形格子的移动消耗倍率。
///
/// 返回移动消耗倍率（1.0 = 标准消耗，2.0 = 双倍消耗）。
/// 返回 `f32::MAX` 表示该地形对该移动类型不可通行。
///
/// # 参数
/// - `terrain_type`: 目标格子的地形类型
/// - `move_category`: 单位的移动类别
pub fn terrain_movement_cost(terrain_type: TerrainType, move_category: MoveCategory) -> f32 {
    match move_category {
        MoveCategory::Walk => walk_cost(terrain_type),
        MoveCategory::Fly => 1.0,
        MoveCategory::Swim => swim_cost(terrain_type),
        MoveCategory::Climb => climb_cost(terrain_type),
        MoveCategory::Teleport => 0.0,
    }
}

/// 步行消耗表。
fn walk_cost(terrain_type: TerrainType) -> f32 {
    match terrain_type {
        TerrainType::Normal => 1.0,
        TerrainType::Highground => 1.5,
        TerrainType::Obstacle => f32::MAX,
        TerrainType::Water => 2.0,
        TerrainType::Bush => 1.5,
        TerrainType::Ice => 2.0,
        TerrainType::Poison => 1.5,
        TerrainType::Burning => 1.5,
        TerrainType::Oil => 1.0,
        TerrainType::Lava => f32::MAX,
    }
}

/// 游泳消耗表。
fn swim_cost(terrain_type: TerrainType) -> f32 {
    match terrain_type {
        TerrainType::Water => 1.0,
        TerrainType::Poison => 1.0,
        _ => 3.0,
    }
}

/// 攀爬消耗表。
fn climb_cost(terrain_type: TerrainType) -> f32 {
    match terrain_type {
        TerrainType::Highground => 1.0,
        TerrainType::Obstacle => 1.0,
        _ => 2.0,
    }
}
