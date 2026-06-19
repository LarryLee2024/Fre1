//! content_logger — 内容加载事件日志 Observer
//!
//! 监听内容/注册表事件（热重载、注册、校验），生成 INFO/WARN 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::infra::registry::registry::OnDefinitionReloaded;
use crate::shared::diagnostics::LogCode;

/// 内容热重载日志 Observer。
pub(crate) fn on_definition_reloaded(trigger: On<OnDefinitionReloaded>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::CNT001,
        event = "definition_reloaded",
        bucket = event.bucket_name,
        version = event.new_version,
        changed = event.changed_ids.len(),
        "definition_reloaded"
    );
}
