/// 运行时 Grid 数据：TerrainGrid + OccupancyGrid
/// 纯数据层，不包含渲染逻辑

/// OccupancyGrid 占用网格（记录哪个单位在哪个格子）
mod occupancy_grid;
/// TerrainGrid 地形网格（存储地形类型 ID）
mod terrain_grid;

pub use occupancy_grid::*;
pub use terrain_grid::*;
