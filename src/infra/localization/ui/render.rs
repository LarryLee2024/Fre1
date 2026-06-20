//! 本地化文本渲染系统
//!
//! 监听 LocalizedText 组件变化，通过 facade 的 resolve_cached 自动更新文本。

use bevy::prelude::*;

use crate::infra::localization::facade::resolve_cached;
use crate::infra::localization::storage::{LocalizationDatabase, LocalizedTextCache};

use super::components::LocalizedText;

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

        match resolve_cached(&db, &mut cache, loc_text.key, &params) {
            Ok(resolved) => {
                text.0 = resolved;
            }
            Err(e) => {
                text.0 = format!("[LOC_ERR: {}]", e);
                warn!(target: "localization", "[Localization] 渲染错误：{}", e);
            }
        }
    }
}
