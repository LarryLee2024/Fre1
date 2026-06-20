//! LocalizedText Component — UI 本地化文本组件
//!
//! UI 系统读取此组件后自动渲染为对应语言文本。
//! 组件本身不存储翻译结果 — 这是缓存层的职责。
//!
//! 详见 `docs/03-technical/localization-design.md` §4

use bevy::prelude::*;

use super::cache::LocalizedTextCache;
use super::database::LocalizationDatabase;

/// UI 组件：本地化文本，携带 Key 和参数
///
/// UI 系统将此组件渲染为对应语言的最终文本。
/// 组件本身不存储翻译结果 — 这是缓存层的职责。
///
/// # 为什么用 &'static str 而非 String？
/// - Key 是编译期已知的常量（由 build.rs 生成），无需运行时分配
/// - 确保所有 key 引用都经过编译检查
#[derive(Component, Debug, Clone)]
pub struct LocalizedText {
    /// Localization Key（编译期常量，来自 generated/keys.rs）
    pub key: &'static str,
    /// Fluent 参数: (参数名, 值) 列表
    /// 参数名 &'static str = 编译期已知
    /// 参数值 String = 运行时动态构建
    pub params: Vec<(&'static str, String)>,
}

impl LocalizedText {
    /// 创建无参数的静态文本
    pub fn static_text(key: &'static str) -> Self {
        Self {
            key,
            params: vec![],
        }
    }

    /// 创建带参数的动态文本
    pub fn with_params(key: &'static str, params: Vec<(&'static str, String)>) -> Self {
        Self { key, params }
    }
}

/// UI 渲染系统：监听 LocalizedText 组件变化，自动更新文本
///
/// 使用 `Changed<LocalizedText>` Filter，只有 key/params 变化时才重新解析。
pub fn render_localized_text(
    db: Res<LocalizationDatabase>,
    mut cache: ResMut<LocalizedTextCache>,
    mut query: Query<(&LocalizedText, &mut Text), Changed<LocalizedText>>,
) {
    for (loc_text, mut text) in query.iter_mut() {
        let params: Vec<(&str, &str)> = loc_text
            .params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        match db.resolve_cached(loc_text.key, &params, &mut cache) {
            Ok(resolved) => {
                text.0 = resolved;
            }
            Err(e) => {
                text.0 = format!("[LOC_ERR: {}]", e);
                warn!("[Localization] 渲染错误：{}", e);
            }
        }
    }
}
