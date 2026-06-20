//! Modal 交互事件
//!
//! 点击模态框的确认/取消按钮时触发对应事件。
//! 遵循 ADR-054 的 Observer 优先原则，监听方通过
//! `app.add_observer(|on: On<ModalConfirmed>| { ... })` 注册。

use bevy::prelude::*;

/// 模态框确认事件
///
/// 由 modal_interaction_observer 在检测到 Confirm 按钮点击时触发。
/// 负载为被关闭的模态框实体。
#[derive(Event, Debug, Clone)]
pub struct ModalConfirmed {
    /// 被关闭的模态框实体
    pub entity: Entity,
}

/// 模态框取消事件
///
/// 由 modal_interaction_observer 在检测到 Cancel 按钮点击时触发。
/// 负载为被关闭的模态框实体。
#[derive(Event, Debug, Clone)]
pub struct ModalCancelled {
    /// 被关闭的模态框实体
    pub entity: Entity,
}
