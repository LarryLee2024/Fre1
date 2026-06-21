//! MapLoaderSystem — MapAsset → GridMap 转换与资源注册
//!
//! 当 MapAsset 完成加载后，将其转换为 GridMap Resource 并注册到世界。
//! 使用 Bevy 0.19 Observer 模式。

use bevy::prelude::*;

use crate::core::domains::tactical::resources::GridMap;

use super::super::asset::MapAsset;
use super::super::events::MapLoadedEvent;
use super::super::loader::{TerrainIndex, convert_to_gridmap};

/// 将已加载的 MapAsset 转换为 GridMap 并注册为 Resource。
///
/// 此系统在 MapAsset 就绪后调用。由于 Bevy Asset 加载是异步的，
/// V1 使用同步路径：调用方确保 MapAsset 已加载。
///
/// 流程:
/// 1. 从 TerrainIndex Resource 获取地形映射
/// 2. 将 MapAsset 转换为 GridMap
/// 3. 插入 GridMap 为 Resource
/// 4. 触发 MapLoadedEvent
///
/// TODO[P3][Map][2026-07-01]: 接入 Bevy AssetEvent，支持异步加载
pub fn load_map_into_world(
    map_asset: &MapAsset,
    commands: &mut Commands,
    terrain_index: &TerrainIndex,
) {
    let grid_map = convert_to_gridmap(map_asset, terrain_index);

    // 插入 GridMap Resource
    commands.insert_resource(grid_map);

    // 触发 MapLoadedEvent
    commands.trigger(MapLoadedEvent {
        map_asset_id: map_asset.metadata.id.clone(),
    });

    info!(target: "map",
        "[Map] 地图 '{}' ({}x{}) 已加载到世界",
        map_asset.metadata.id,
        map_asset.metadata.width,
        map_asset.metadata.height,
    );
}
