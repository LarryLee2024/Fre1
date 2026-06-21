//! MapAsset — 运行时地图资产类型
//!
//! MapAsset 是 Bevy Asset（`#[derive(Asset, TypePath)]`），
//! 由 Importer 从 Tiled TMX 转换而来，存储在 `assets/config/04_world/maps/` 目录。
//!
//! 设计原则：
//! - 不可变、版本可控、不包含业务逻辑
//! - Tile 只存 TerrainId（String），Gameplay 数值从 TerrainDef Registry 查询
//! - Object Layer 是一等公民，Object 是定义而非实例
//!
//! 详见 ADR-065 §3 和 docs/03-content/definitions/world/map-def.md

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::shared::localization_key::LocalizationKey;

use super::types::{
    MapGridLayout, MapHexDirection, MapObjectGuid, MapTileFlags, ObjectShape, PropertyMap,
};

// ─── MapAsset ──────────────────────────────────────────────────────

/// 运行时地图资产——Importer 从 Tiled TMX 转换而来。
///
/// MapAsset 是 L4 World 层的运行时地图数据。
/// 不可变、版本可控、不包含业务逻辑。
/// 不是 DefRegistry 成员——通过 Bevy AssetServer 加载。
///
/// 🟥 禁止运行时修改。
/// 🟥 禁止承载业务逻辑（寻路、战斗结算等）。
#[derive(Asset, TypePath, Deserialize, Serialize, Clone, Debug)]
pub struct MapAsset {
    /// 地图元数据（标识、尺寸、布局）
    pub metadata: MapMetadata,

    /// 地形网格数据（核心数据）
    pub terrain_grid: TerrainGrid,

    /// 对象层列表（一等公民）
    #[serde(default)]
    pub object_layers: Vec<ObjectLayer>,

    /// 出生点列表
    #[serde(default)]
    pub spawn_points: Vec<SpawnPoint>,

    /// 区域/命名网格范围集合
    #[serde(default)]
    pub regions: Vec<MapRegion>,

    /// 通行性导航掩码（Importer 预计算，运行时只读）
    pub navigation_mask: NavigationMask,
}

// ─── MapMetadata ───────────────────────────────────────────────────

/// 地图元数据——标识和尺寸信息。
///
/// 所有像素字段由 Importer 从 TMX 元数据复制。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapMetadata {
    /// 地图 Def ID（全局唯一，前缀: `map:`）
    pub id: String,

    /// 显示名称本地化 Key
    pub name_key: LocalizationKey,

    /// Schema 版本号（当前版本: 1）
    pub schema_version: u32,

    /// 网格宽度（格子数），> 0
    pub width: u32,

    /// 网格高度（格子数），> 0
    pub height: u32,

    /// 网格布局类型
    pub layout: MapGridLayout,

    /// Tiled 原始像素宽度
    pub pixel_width: u32,

    /// Tiled 原始像素高度
    pub pixel_height: u32,

    /// 每格像素宽度
    pub tile_width: u32,

    /// 每格像素高度
    pub tile_height: u32,
}

// ─── TerrainGrid & TileEntry ──────────────────────────────────────

/// 地形网格——地图的核心数据结构。
///
/// 按行优先布局。总格子数 = width * height。
/// 索引公式: index = y * width + x
///
/// 🟥 禁止在此结构中存储 Gameplay 数值。
/// 所有数值从 TerrainDef Registry 查询。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TerrainGrid {
    /// 宽度（格数），必须与 MapMetadata.width 一致
    pub width: u32,

    /// 高度（格数），必须与 MapMetadata.height 一致
    pub height: u32,

    /// 按行优先排列的 Tile 数据
    pub tiles: Vec<TileEntry>,
}

/// 单个 Tile 的运行时数据。
///
/// 以人类可读的 String terrain_id 存储，而非 packed u32。
/// 🟥 禁止存储 Gameplay 数值。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TileEntry {
    /// 地形类型 ID（String，如 `"ter:plain"`, `"ter:forest"`）
    pub terrain_id: String,

    /// 高度（0-255）
    pub height: u8,

    /// 位标记——PASSABLE, FLYABLE, BUILDABLE, BLOCKS_SIGHT
    pub flags: MapTileFlags,

    /// 旋转（0-3，90 度递增）
    pub rotation: u8,

    /// Tiled 原始 GID（仅用于调试追溯）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
}

// ─── ObjectLayer & MapObject ───────────────────────────────────────

/// 对象层——地图上的一层对象定义。
///
/// 对象层是 MapAsset 的一等公民。
/// 运行时由 ObjectInstantiator 根据 class 映射策略实例化 ECS Entity。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ObjectLayer {
    /// Tiled 原始层 ID（仅用于调试追溯）
    pub id: u32,

    /// 层名称
    pub name: String,

    /// 透明度
    pub opacity: f32,

    /// 是否可见
    pub visible: bool,

    /// 层像素偏移 X
    pub offset_x: i32,

    /// 层像素偏移 Y
    pub offset_y: i32,

    /// 本层所有对象（携带稳定 GUID）
    pub objects: Vec<MapObject>,
}

/// 地图对象——运行时 ECS Entity 的模板定义。
///
/// Object 不是 Entity。实例化由 Domain System 通过 class 类型决定。
/// 稳定 GUID 保证跨存档/跨场景的身份追溯。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapObject {
    /// 稳定 GUID（Importer 内容哈希生成）
    pub guid: MapObjectGuid,

    /// Tiled 原始 ID（仅用于调试追溯）
    pub tiled_id: u32,

    /// 对象名称（Tiled 中的 name 字段）
    pub name: String,

    /// 对象类型/Custom Class（Tiled Class 名）
    pub class: String,

    /// 网格位置（Importer 从像素坐标转换）
    pub position: MapObjectPos,

    /// 尺寸（格）
    pub width: u32,

    /// 尺寸（格）
    pub height: u32,

    /// 旋转角度（度）
    pub rotation: f32,

    /// 自定义属性映射
    pub properties: PropertyMap,

    /// 形状（用于碰撞/区域判定）
    pub shape: ObjectShape,
}

/// MapObject 使用的简化位置类型。
///
/// 避免 MapAsset 层直接依赖 tactical domain 的 GridPos。
/// Loader 负责在运行时转换为 GridPos。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapObjectPos {
    pub x: i32,
    pub y: i32,
}

impl MapObjectPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

// ─── SpawnPoint ────────────────────────────────────────────────────

/// 出生点——单位生成位置。
///
/// 通过 spawn_group_id 引用 L3 SpawnGroupDef，遵循 L4 → L3 的合法引用方向。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SpawnPoint {
    /// 稳定 GUID
    pub guid: MapObjectGuid,

    /// 生成组 ID（引用 L3 SpawnGroupDef）
    pub spawn_group_id: String,

    /// 网格位置
    pub position: MapObjectPos,

    /// 阵营（引用 L0 FactionDef）
    pub faction: Option<String>,

    /// 朝向
    pub facing: MapHexDirection,

    /// 额外属性
    #[serde(default)]
    pub properties: PropertyMap,
}

// ─── MapRegion ─────────────────────────────────────────────────────

/// 区域——地图上的命名 Tile 集合。
///
/// v1 仅做数据存储，不提供运行时 Region 查询 API。
/// 数据基础为未来 Region 系统预留。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapRegion {
    /// 区域标识符（字符串 ID，同一地图内唯一）
    pub id: String,

    /// 显示名称本地化 Key
    pub name_key: LocalizationKey,

    /// 包含的网格位置集合
    pub tiles: Vec<MapObjectPos>,

    /// 区域属性
    #[serde(default)]
    pub properties: PropertyMap,
}

// ─── NavigationMask ────────────────────────────────────────────────

/// 通行性导航掩码——Importer 在构建时预计算。
///
/// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性。
/// 运行时作为 GridMap 寻路的加速结构。
///
/// 🟥 此字段由 Importer 生成，内容创作者不应手动编写。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NavigationMask {
    /// 掩码宽度（格数）
    pub width: u32,

    /// 掩码高度（格数）
    pub height: u32,

    /// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性
    ///
    /// 位定义：
    ///   位 0: WALK — 地面单位可通行 (0x01)
    ///   位 1: FLY — 飞行单位可通行 (0x02)
    ///   位 2: SWIM — 游泳单位可通行 (0x04)
    ///   位 3-7: 保留
    pub data: Vec<u8>,
}
