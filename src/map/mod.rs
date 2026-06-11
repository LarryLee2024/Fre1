// 地图模块：网格系统、地形数据、寻路、运行时 Grid
// Terrain 枚举和 Tile 组件已删除，地形数据由 TerrainGrid 纯数据存储

mod data;
mod grid;
mod hp_bar;
mod pathfinding;
pub mod runtime;

use bevy::prelude::*;

// 公共 re-exports（data 和 runtime 的类型通过 * 导出，外部用 crate::map::TerrainRegistry 即可）
pub use data::*;
pub use grid::*;
pub use pathfinding::*;
pub use runtime::*;

/// 地图插件（组合 MapGrid + MapData + Runtime 子插件）
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((data::MapDataPlugin, grid::MapGridPlugin))
            .insert_resource(pathfinding::TerrainCostRegistry::default())
            .insert_resource(runtime::OccupancyGrid::default())
            // 注册 Reflect 类型
            .register_type::<grid::GameMap>()
            .register_type::<runtime::OccupancyGrid>()
            .register_type::<runtime::TerrainGrid>()
            .add_systems(Update, (runtime::update_occupancy_grid, hp_bar::update_hp_bars));
    }
}
