/// AssetsPlugin：资源管理插件
///
/// 负责加载字体等全局共享资源。
/// 从 src/assets.rs 迁移至此（Phase 4.1）。
use bevy::prelude::*;

use super::game_assets::{CN_FONT, CnFont};

/// 资源管理插件
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let cn_font = CnFont::from(asset_server);
        let handle = cn_font.as_handle().clone();
        app.insert_resource(cn_font);
        app.add_systems(
            PostStartup,
            move |asset_server: Res<AssetServer>| match asset_server.load_state(&handle) {
                bevy::asset::LoadState::Loaded => {
                    bevy::log::info!(
                        target: "assets",
                        event = "fonts_loaded",
                        font = CN_FONT,
                        "字体资源已加载"
                    );
                }
                bevy::asset::LoadState::Failed(err) => {
                    bevy::log::error!(
                        target: "assets",
                        event = "asset_load_failed",
                        path = CN_FONT,
                        error = %err,
                        "字体资源加载失败"
                    );
                }
                _ => {}
            },
        );
    }
}
