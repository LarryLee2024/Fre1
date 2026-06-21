//! MapPlugin — 地图管线插件
//!
//! 注册地图管线的 ECS 基础设施：
//! - MapAsset Bevy Asset 类型
//! - TerrainIndex Resource
//! - MapRenderConfig Resource
//! - MapLoadedEvent / MapUnloadedEvent
//!
//! 详见 ADR-065 §5.3

use bevy::app::Plugin;
use bevy::prelude::*;

use super::asset::MapAsset;
use super::loader::TerrainIndex;
use super::renderer::spawn::MapRenderConfig;

/// 地图管线插件。
///
/// 注册在 Phase 8 (Infrastructure)，位于 CameraPlugin 之后。
///
/// V1 注册:
/// - MapAsset (Bevy Asset 类型，通过 AssetServer 异步加载)
/// - TerrainIndex Resource (terrain_id → u16 索引映射)
/// - MapRenderConfig Resource (渲染配置)
/// - MapLoadedEvent / MapUnloadedEvent (生命周期事件)
///
/// TODO[P3][Map][2026-07-01]: 注册 Observer 系统处理 MapUnloadedEvent
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Asset 类型 ──
        app.init_asset::<MapAsset>();

        // ── 注册 Resources ──
        app.init_resource::<TerrainIndex>();
        app.init_resource::<MapRenderConfig>();

        // ── Events（Bevy 0.19 通过 #[derive(Event)] 自动注册）──

        info!(target: "map", "[MapPlugin] 地图管线已注册");
    }
}

// Events MapLoadedEvent / MapUnloadedEvent 在 events.rs 中通过
// #[derive(Event)] 自动注册，无需手动 add_event。
// 详见 Bevy 0.19 迁移指南。
