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
        // 延迟到 Startup 系统初始化 CnFont，确保 AssetServer 已就绪
        app.add_systems(
            Startup,
            |mut commands: Commands, asset_server: Res<AssetServer>| {
                commands.insert_resource(CnFont::from(&asset_server));
                bevy::log::info!(
                    target: "assets",
                    event = "fonts_loaded",
                    font = CN_FONT,
                    "字体资源已加载"
                );
            },
        );
    }
}
