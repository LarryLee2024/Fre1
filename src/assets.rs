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

/// 初始化中文字体资源
///
/// 字体加载回退策略（当前版本）：
/// 1. 优先加载项目自带字体 fonts/Arial Unicode.ttf
/// 2. 若加载失败，Bevy 会使用内置默认字体渲染（不支持中文）
///
/// 未来迁移路径（Bevy 0.19+）：
/// - 使用 system_font_discovery 自动发现系统中文字体
/// - 回退链：系统字体 → 项目自带字体 → Bevy 默认字体
/// - 示例：asset_server.system_fonts().find_by_name("PingFang SC")
pub fn init_cn_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CnFont::from(&asset_server));
}

/// 资源管理插件
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_cn_font);
    }
}
