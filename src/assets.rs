// 资源管理模块：全局共享资源（字体、常量等）

use bevy::prelude::*;

/// 中文字体路径
const CN_FONT: &str = "fonts/Arial Unicode.ttf";

/// 中文字体资源（全局预加载，避免重复加载）
#[derive(Resource)]
pub struct CnFont {
    pub handle: Handle<Font>,
}

impl CnFont {
    pub fn from(asset_server: &AssetServer) -> Self {
        Self {
            handle: asset_server.load(CN_FONT),
        }
    }
}

/// 初始化中文字体资源
pub fn init_cn_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CnFont::from(&asset_server));
}
