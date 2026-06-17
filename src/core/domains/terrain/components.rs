//! ECS Components — 地形领域组件
//!
//! 定义地形属性、表面覆盖、地形效果等 ECS 组件。
//! 详见 docs/02-domain/domains/terrain_domain.md
//! 详见 docs/04-data/domains/terrain_schema.md

use std::collections::HashSet;

use bevy::prelude::*;

// ─── 枚举类型 ─────────────────────────────────────────────────────

/// 地形类型（Definition — 静态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum TerrainType {
    /// 平地，无特殊效果
    Normal,
    /// 高地，提供视野/命中优势
    Highground,
    /// 障碍，不可通行，提供掩体
    Obstacle,
    /// 水域，通行消耗翻倍
    Water,
    /// 丛林，提供隐蔽
    Bush,
    /// 冰面，移动消耗翻倍
    Ice,
    /// 毒池，进入施加中毒
    Poison,
    /// 灼烧，每回合燃烧伤害
    Burning,
    /// 油面，可被点燃
    Oil,
    /// 岩浆，极高伤害
    Lava,
}

impl TerrainType {
    /// 所有 TerrainType 变体，用于枚举迭代。
    pub const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Highground,
        Self::Obstacle,
        Self::Water,
        Self::Bush,
        Self::Ice,
        Self::Poison,
        Self::Burning,
        Self::Oil,
        Self::Lava,
    ];
}

/// 基础通行性（Definition — 静态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum Passability {
    /// 可行走
    Walkable,
    /// 阻挡（障碍/边界）
    Blocked,
    /// 飞行单位可越过
    Flyable,
    /// 所有单位不可通行
    Impassable,
}

impl Passability {
    /// 是否允许地面单位通行。
    pub fn is_walkable(&self) -> bool {
        matches!(self, Self::Walkable | Self::Flyable)
    }

    /// 是否允许飞行单位通行。
    pub fn is_flyable(&self) -> bool {
        matches!(self, Self::Walkable | Self::Flyable)
    }
}

/// 基础遮蔽度（Definition — 静态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum Concealment {
    /// 完全可见
    None,
    /// 半遮蔽，隐蔽 -2 命中
    Half,
    /// 不可见，无法作为目标
    Full,
}

/// 表面类型 — 描述格子表面的物质状态（Instance — 运行时可变）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SurfaceType {
    /// 正常表面
    Normal,
    /// 冰面
    Ice,
    /// 油面
    Oil,
    /// 水面
    Water,
    /// 毒池
    Poison,
    /// 灼烧
    Burning,
    /// 岩浆
    Lava,
}

/// 表面恢复方式。所有表面变化必须有对应的恢复机制（不变量 3.3）。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SurfaceRecovery {
    /// 到期自动恢复
    Timed { total_duration: u32 },
    /// 被驱散恢复
    Dispel,
    /// 显式声明为永久变化（需 [Data Exemption]）
    Permanent,
}

// ─── 坐标类型 ─────────────────────────────────────────────────────

/// 地形网格坐标 — Terrain 域专用位置类型。
///
/// 与 Tactical 域的 GridPos 结构对齐，但保持域间独立（Data Law 012）。
/// 可通过 From 转换与 tactical 的 GridPos 互转。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
    pub layer: i8,
}

impl TilePos {
    /// 创建新坐标（默认 layer=0）。
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y, layer: 0 }
    }

    /// 创建带层高的坐标。
    pub const fn with_layer(x: i32, y: i32, layer: i8) -> Self {
        Self { x, y, layer }
    }

    /// 曼哈顿距离（Square 网格）。
    pub fn manhattan_distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    /// 检查两个 TilePos 是否指向同一格。
    pub fn is_same_tile(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.layer == other.layer
    }
}

// ─── ECS Components ───────────────────────────────────────────────

/// 格子的地形属性集合。
///
/// Definition 层定义静态配置（terrain_type, base_passability, base_concealment），
/// 运行时表面可被修改（surface, original_surface）。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct TileProperties {
    /// 地形类型（Definition — 静态）
    pub terrain_type: TerrainType,
    /// 层高（0 = 地面层，1+ = 高地层）
    pub height: i32,
    /// 基础通行性（Definition — 静态）
    pub base_passability: Passability,
    /// 基础遮蔽度（Definition — 静态）
    pub base_concealment: Concealment,
    /// 当前表面类型（Instance — 运行时可变）
    pub surface: SurfaceType,
    /// 原始表面类型（Instance — 用于恢复）
    pub original_surface: SurfaceType,
    /// 附加标签（用于效果匹配和查询）
    pub tags: Vec<String>,
}

impl TileProperties {
    /// 创建新的 TileProperties（surface 与 original_surface 相同）。
    pub fn new(
        terrain_type: TerrainType,
        height: i32,
        base_passability: Passability,
        base_concealment: Concealment,
        surface: SurfaceType,
    ) -> Self {
        Self {
            terrain_type,
            height,
            base_passability,
            base_concealment,
            surface,
            original_surface: surface,
            tags: Vec::new(),
        }
    }

    /// 当前通行性（由 surface 和 terrain_type 共同决定）。
    ///
    /// 表面类型的通行性影响：
    /// - Lava/Water → 不可通行
    /// - 其他表面 → 沿用 base_passability
    pub fn current_passability(&self) -> Passability {
        match self.surface {
            SurfaceType::Lava | SurfaceType::Water => Passability::Impassable,
            _ => self.base_passability,
        }
    }

    /// 当前遮蔽度（由 surface 和 terrain_type 共同决定）。
    ///
    /// 表面类型的遮蔽度影响：
    /// - Burning → 半遮蔽（烟雾）
    /// - Poison → 半遮蔽（毒气）
    /// - 其他表面 → 沿用 base_passability
    ///
    /// 地形类型的遮蔽度影响：
    /// - Bush → 半遮蔽
    pub fn current_concealment(&self) -> Concealment {
        // 表面变化优先——燃烧/毒气产生遮蔽
        match self.surface {
            SurfaceType::Burning | SurfaceType::Poison => return Concealment::Half,
            _ => {}
        }
        // 地形类型提供遮蔽
        match self.terrain_type {
            TerrainType::Bush => Concealment::Half,
            _ => self.base_concealment,
        }
    }
}

/// 表面类型的变化记录。每个表面变化必须可恢复（不变量 3.3）。
///
/// 当格子表面被覆盖时附加此组件，恢复后移除。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SurfaceOverride {
    /// 当前表面类型
    pub current: SurfaceType,
    /// 原始表面类型（用于恢复）
    pub original: SurfaceType,
    /// 该覆盖的剩余持续回合数（None = 永久）
    pub remaining_duration: Option<u32>,
    /// 恢复方式
    pub recovery: SurfaceRecovery,
}

impl SurfaceOverride {
    /// 创建定时恢复的表面覆盖。
    pub fn timed(current: SurfaceType, original: SurfaceType, duration: u32) -> Self {
        Self {
            current,
            original,
            remaining_duration: Some(duration),
            recovery: SurfaceRecovery::Timed {
                total_duration: duration,
            },
        }
    }

    /// 创建可驱散的表面覆盖。
    pub fn dispel(current: SurfaceType, original: SurfaceType) -> Self {
        Self {
            current,
            original,
            remaining_duration: None,
            recovery: SurfaceRecovery::Dispel,
        }
    }

    /// 创建永久表面覆盖（需谨慎使用）。
    pub fn permanent(current: SurfaceType, original: SurfaceType) -> Self {
        Self {
            current,
            original,
            remaining_duration: None,
            recovery: SurfaceRecovery::Permanent,
        }
    }

    /// 是否已到期（剩余回合为 0）。
    pub fn is_expired(&self) -> bool {
        matches!(self.remaining_duration, Some(0))
    }
}

/// 绑定到格子的地形效果记录。
///
/// Terrain 领域只记录「哪个格子挂了哪些效果」，
/// 效果的实际生命周期由 Effect 领域管理。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct TerrainAttachEffect {
    /// 绑定的格子位置
    pub tile: TilePos,
    /// 引用的 EffectDefId
    ///
    /// TODO[P2][Terrain]: 迁移到 DefinitionId（需 DefinitionId 实现 Reflect 或移除 #[reflect(Component)]）
    pub effect_id: String,
    /// 剩余持续时间（回合数），None = 永久
    pub remaining_duration: Option<u32>,
}

/// 陷阱消耗状态 — 记录哪些消耗型陷阱已被触发。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct HazardTriggeredState {
    /// 已消耗的陷阱 ID 集合
    pub consumed_hazards: HashSet<String>,
}

impl HazardTriggeredState {
    /// 创建空的消耗状态。
    pub fn new() -> Self {
        Self {
            consumed_hazards: HashSet::new(),
        }
    }

    /// 检查陷阱是否已被消耗。
    pub fn is_consumed(&self, hazard_id: &str) -> bool {
        self.consumed_hazards.contains(hazard_id)
    }

    /// 标记陷阱为已消耗。
    pub fn consume(&mut self, hazard_id: String) {
        self.consumed_hazards.insert(hazard_id);
    }
}

impl Default for HazardTriggeredState {
    fn default() -> Self {
        Self::new()
    }
}
