use super::data::MapDataPlugin;
use super::grid::MapGridPlugin;
use super::pathfinding::{TerrainCostRegistry, TerrainMapCache, cache_terrain_map};
use bevy::prelude::*;

/// 地形缓存为空时需要刷新
fn terrain_cache_needs_refresh(cache: Res<TerrainMapCache>) -> bool {
    cache.map.is_empty()
}

/// 地图插件（组合 MapGrid + MapData 子插件）
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapDataPlugin, MapGridPlugin))
            .insert_resource(TerrainCostRegistry::default())
            .insert_resource(TerrainMapCache::default())
            .add_systems(
                Update,
                cache_terrain_map.run_if(terrain_cache_needs_refresh),
            );
    }
}
