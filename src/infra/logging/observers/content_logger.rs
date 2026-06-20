//! content_logger — 内容加载事件日志 Observer
//!
//! 监听内容/注册表事件（热重载、注册、校验），生成 INFO/WARN 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::infra::logging::metrics;
use crate::infra::registry::registry::OnDefinitionReloaded;
use crate::shared::diagnostics::LogCode;

/// 内容热重载日志 Observer。
#[tracing::instrument(skip_all, target = "content", fields(
    code = ?LogCode::CNT001,
    event = "content_loaded",
))]
pub(crate) fn on_definition_reloaded(trigger: On<OnDefinitionReloaded>) {
    metrics::record(LogCode::CNT001);
    let event = trigger.event();
    info!(
        target = "content",
        bucket = event.bucket_name,
        version = event.new_version,
        changed = event.changed_ids.len(),
        "内容重载",
    );
}
