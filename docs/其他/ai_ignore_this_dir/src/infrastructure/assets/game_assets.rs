/// 全局共享资源：字体、常量等
///
/// 从 src/assets.rs 迁移至 infrastructure/ 层（Phase 4.1）。
/// 资源加载属于基础设施职责，负责管理应用程序级别的共享资源。
use bevy::prelude::*;

/// 中文字体路径
pub(crate) const CN_FONT: &str = "fonts/Arial Unicode.ttf";

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
