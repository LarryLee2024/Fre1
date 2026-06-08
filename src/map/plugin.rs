use super::grid::MapGridPlugin;
use super::data::MapDataPlugin;
use bevy::prelude::*;

/// 地图插件（组合 MapGrid + MapData 子插件）
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MapDataPlugin,
            MapGridPlugin,
        ));
    }
}
