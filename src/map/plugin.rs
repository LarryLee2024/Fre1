use super::data::MapDataPlugin;
use super::grid::MapGridPlugin;
use super::pathfinding::TerrainCostRegistry;
use super::runtime::{OccupancyGrid, update_occupancy_grid};
use bevy::prelude::*;

/// 地图插件（组合 MapGrid + MapData + Runtime 子插件）
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapDataPlugin, MapGridPlugin))
            .insert_resource(TerrainCostRegistry::default())
            .insert_resource(OccupancyGrid::default())
            .add_systems(Update, update_occupancy_grid);
    }
}
