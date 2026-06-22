//! content_logger — 内容加载事件日志 Observer
//!
//! 监听内容/注册表事件（热重载、注册、校验），生成 INFO/WARN 日志。
//!
//! # 日志模式说明（按项目日志规则 .trae/rules/日志规则.md）
//!
//! 本文件属于**模式 A — 领域事件 Observer**：
//! - 通过 `On<T>` 监听领域事件（这里监听 `OnDefinitionReloaded`）
//! - 使用 `#[tracing::instrument]` + `emit_info!` 宏
//! - `target: "content"` 指定路由
//! - span 放不变量（code、event），`emit_info!` 只放变量
//!
//! ## 与 content_plugin.rs 的关系（不矛盾）
//!
//! 本 Observer 负责**注册表层面的热重载事件日志**（CNT001），
//! content_plugin.rs 的直接日志负责**文件加载阶段的运行状态输出**。
//! 两者互补：前者是业务事件驱动，后者是加载器运行状态。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::emit_info;
use crate::infra::registry::registry::OnDefinitionReloaded;
use crate::shared::diagnostics::LogCode;

/// 内容热重载日志 Observer。
#[tracing::instrument(skip_all, target = "content", fields(
    code = ?LogCode::CNT001,
    event = "content_loaded",
))]
pub(crate) fn on_definition_reloaded(trigger: On<OnDefinitionReloaded>) {
    let event = trigger.event();
    emit_info!(
        LogCode::CNT001,
        bucket = event.bucket_name,
        version = event.new_version,
        changed = event.changed_ids.len(),
        "内容重载",
    );
}
