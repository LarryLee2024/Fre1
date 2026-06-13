// 资源管理模块：全局共享资源（字体、常量等）

use bevy::prelude::*;

/// 中文字体路径
const CN_FONT: &str = "fonts/Arial Unicode.ttf";

/// 中文字体资源（全局预加载，避免重复加载）
///
/// 统一字体获取接口，为未来迁移 system_font_discovery 预留扩展点。
/// Bevy 0.19 将提供 system_font_discovery 特性，可自动发现系统字体，
/// 届时 CnFont 可改为优先使用系统字体，项目自带字体作为回退。
#[derive(Resource)]
pub struct CnFont {
    handle: Handle<Font>,
}

impl CnFont {
    pub fn from(asset_server: &AssetServer) -> Self {
        Self {
            handle: asset_server.load(CN_FONT),
        }
    }

    /// 获取中文字体 Handle（克隆）
    pub fn font_handle(&self) -> Handle<Font> {
        self.handle.clone()
    }

    /// 获取中文字体 Handle 引用
    pub fn as_handle(&self) -> &Handle<Font> {
        &self.handle
    }

    /// 创建 TextFont 使用中文字体
    pub fn text_font(&self, size: f32) -> TextFont {
        TextFont {
            font: self.handle.clone(),
            font_size: size,
            ..default()
        }
    }
}

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
