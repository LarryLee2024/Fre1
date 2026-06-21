//! Map — 地图管线模块（L2 Infra）
//!
//! 提供完整的地图资产加载管线：MapAsset (Bevy Asset) → Loader → GridMap Resource。
//! Tiled TMX 在构建时由独立工具 `tools/map_importer/` 转换为 MapAsset (RON)。
//!
//! 模块结构：
//! - types: 辅助类型（MapObjectGuid, MapTileFlags, MapGridLayout 等）
//! - asset: MapAsset Bevy Asset 类型定义（不可变，版本可控）
//! - events: MapLoadedEvent / MapUnloadedEvent
//! - importer: 构建时 TMX → MapAsset 转换的纯函数（运行时不需要）
//! - loader: 运行时 MapAsset → GridMap 转换
//! - renderer: V1 Entity-per-Tile 快速渲染路径
//! - systems: ECS 系统（MapAsset 加载/卸载 → GridMap 资源管理）
//!
//! 详见 ADR-065 (Map 内容管线架构)

mod asset;
mod events;
pub mod importer;
mod loader;
pub mod renderer;
pub mod systems;
mod types;

mod plugin;

pub use asset::{
    MapAsset, MapMetadata, MapObject, MapObjectPos, MapRegion, NavigationMask, ObjectLayer,
    SpawnPoint, TerrainGrid, TileEntry,
};
pub use events::{MapLoadedEvent, MapUnloadedEvent};
pub use loader::{TerrainIndex, build_terrain_index, convert_to_gridmap};
pub use plugin::MapPlugin;
pub use types::{
    MapGridLayout, MapHexDirection, MapObjectGuid, MapTileFlags, ObjectShape, PropertyMap,
    PropertyValue,
};
