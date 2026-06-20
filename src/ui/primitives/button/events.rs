//! Button 交互事件
//!
//! 点击按钮时触发 ButtonClicked 事件，其他系统通过 Observer 监听。
//! 遵循 ADR-054 的 trigger() + Observer 优先原则。

use bevy::prelude::*;

/// 按钮点击事件
///
/// 由 button_interaction_system 在检测到点击释放时触发。
/// 监听方通过 `app.observe(|trigger: Trigger<ButtonClicked>| { ... })` 注册。
#[derive(Event, Debug, Clone)]
pub struct ButtonClicked {
    /// 被点击的按钮实体
    pub entity: Entity,
}
