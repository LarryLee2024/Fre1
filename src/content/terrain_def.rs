//! TerrainDef — L0 Vocabulary 地形类型定义
//!
//! 定义游戏中一种基础地形的通行属性、战术加成和表现元数据。
//! 从 assets/config/terrains/*.ron 加载，是 Tile 数据的数值来源。
//!
//! 与 terrain domain 的 TileProperties/TerrainType 的关系：
//!   TerrainDef (Content L0) → 加载时转换为 Domain 数据
//!   TileEntry.terrain_id → TerrainDef.id → TileProperties/TerrainType
//!
//! 详见 docs/03-content/definitions/vocabulary/terrain-def.md

use serde::{Deserialize, Serialize};

use crate::shared::localization_key::LocalizationKey;

/// 地形类型定义——游戏中一种基础地形的通行属性、战术加成和表现元数据。
///
/// TerrainDef 是 L0 Vocabulary Def，禁止引用任何其他 Def 类型。
/// 地形的 Gameplay 数值在 TerrainDef 中定义，Tile 只存储 TerrainId 引用。
///
/// 详见 ADR-065 §4 Tile → Config 映射策略。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerrainDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（前缀: `ter:`）
    pub id: String,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 通行属性 ──
    /// 移动消耗（格数），0.0 = 不可通行
    pub move_cost: f32,
    /// 飞行消耗（可选），None = 与 move_cost 相同
    pub fly_cost: Option<f32>,

    // ── 战术属性 ──
    /// 防御加成（被攻击时额外防御值）
    pub defense_bonus: i32,
    /// 闪避加成（站在该地形上获得的额外闪避率）
    pub avoid_bonus: i32,
    /// 基础遮蔽度
    pub concealment: Concealment,
    /// 通行标记位集合
    pub flags: TerrainFlags,

    // ── 表现资源 ──
    /// 地形颜色（十六进制 RGB，如 "#8FBA59"）
    pub color_hex: Option<String>,
    /// 地形瓦片材质 Key（渲染系统索引用）
    pub tile_material_key: Option<String>,
}

/// 地形遮蔽度——影响单位在该地形上的被侦测/瞄准难度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Concealment {
    /// 无遮蔽——完全可见
    None,
    /// 半遮蔽——隐蔽 -2 命中修正
    Half,
    /// 全遮蔽——不可见，无法作为目标
    Full,
}

/// 地形通行标记集合——决定该地形的基本通行方式。
///
/// 这些标记在 Map Importer 阶段被转换为 TileData.flags（packed u8），
/// 运行时不再查询 TerrainDef。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct TerrainFlags {
    /// 可行走——地面单位可通过
    pub passable: bool,
    /// 可飞行——飞行单位可通过
    pub flyable: bool,
    /// 可建造——可在该地形上建造/放置设施
    pub buildable: bool,
    /// 阻挡视线——该地形阻挡射击线/视野
    pub blocks_sight: bool,
}

impl TerrainFlags {
    /// 所有位标记均为 false 的默认值。
    pub const fn empty() -> Self {
        Self {
            passable: false,
            flyable: false,
            buildable: false,
            blocks_sight: false,
        }
    }
}

impl Default for TerrainFlags {
    fn default() -> Self {
        Self::empty()
    }
}
