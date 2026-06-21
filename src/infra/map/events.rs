//! Events — 地图生命周期事件
//!
//! 地图加载和卸载时触发的事件。
//! 使用 Bevy 0.19 trigger() + Observer 模式。

use bevy::prelude::*;

/// 地图加载完成事件——当 MapAsset 完全加载并转换为 ECS 状态后触发。
#[derive(Debug, Clone, Event)]
pub struct MapLoadedEvent {
    /// 加载的地图 Asset ID
    pub map_asset_id: String,
}

/// 地图卸载事件——地图场景退出时触发。
#[derive(Debug, Clone, Event)]
pub struct MapUnloadedEvent {
    /// 卸载的地图 Asset ID
    pub map_asset_id: String,
}
