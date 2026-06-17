//! ErrorMonitor——游戏错误监控系统
//!
//! 监听 GameErrorMessage，输出 ERROR 级别 tracing 日志。
//! 后续可扩展：UI Toast 通知、错误计数、错误率告警。

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use tracing::error;

use super::error_event::GameErrorMessage;

/// 错误监控系统——消费 GameErrorMessage
///
/// 1. 输出结构化 ERROR 级别日志
/// 2. (未来) 将 DomainError/Bug 类型错误转发到 UI Toast
///
/// 使用固定 target "game_error"，source 放在字段中
/// （tracing macro 的 target 必须为静态字符串）
pub fn error_monitor(mut reader: MessageReader<GameErrorMessage>) {
    for msg in reader.read() {
        let type_label = msg.error_type.label();

        if msg.context.is_empty() {
            error!(
                target: "game_error",
                error_type = type_label,
                source = msg.source,
                "{}", msg.message,
            );
        } else {
            let ctx: Vec<(&str, &str)> =
                msg.context.iter().map(|(k, v)| (*k, v.as_str())).collect();
            error!(
                target: "game_error",
                error_type = type_label,
                source = msg.source,
                context = ?ctx,
                "{}", msg.message,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::error_event::{ErrorType, GameErrorMessage};

    /// 验证 error_monitor 能处理空上下文消息
    #[test]
    fn monitor_消费空上下文消息() {
        let mut app = App::new();
        app.add_message::<GameErrorMessage>()
            .add_systems(Update, error_monitor);

        app.world_mut().write_message(GameErrorMessage::new(
            ErrorType::DomainError,
            "测试错误",
            "test",
        ));

        app.update(); // 不应 panic
    }

    /// 验证 error_monitor 能处理带上下文消息
    #[test]
    fn monitor_消费带上下文消息() {
        let mut app = App::new();
        app.add_message::<GameErrorMessage>()
            .add_systems(Update, error_monitor);

        app.world_mut().write_message(
            GameErrorMessage::new(ErrorType::Bug, "非法状态", "test")
                .with_context("state", "conflict"),
        );

        app.update(); // 不应 panic
    }

    /// 验证多个消息同时消费
    #[test]
    fn monitor_消费批量消息() {
        let mut app = App::new();
        app.add_message::<GameErrorMessage>()
            .add_systems(Update, error_monitor);

        app.world_mut().write_message(GameErrorMessage::new(
            ErrorType::Infrastructure,
            "IO 错误",
            "test",
        ));
        app.world_mut()
            .write_message(GameErrorMessage::new(ErrorType::Bug, "空指针", "test"));

        app.update(); // 不应 panic
    }
}
