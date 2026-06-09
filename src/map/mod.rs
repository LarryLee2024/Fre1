// 地图模块：网格系统、地形数据、寻路、运行时 Grid
// Terrain 枚举和 Tile 组件已删除，地形数据由 TerrainGrid 纯数据存储

mod data;
mod grid;
mod pathfinding;
mod plugin;
pub mod runtime;

// 公共 re-exports（data 和 runtime 的类型通过 * 导出，外部用 crate::map::TerrainRegistry 即可）
pub use data::*;
pub use grid::*;
pub use pathfinding::*;
pub use plugin::MapPlugin;
pub use runtime::*;
