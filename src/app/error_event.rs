//! GameErrorMessage——跨层统一错误上报通道
//!
//! 任一 System 遇到不可恢复的异常时发送此 Message：
//! 1. ErrorMonitor 消费 → ERROR 级别 tracing 日志
//! 2. UI 层消费 → 显示 Toast 通知（仅对玩家可见错误）
//!
//! 🟥 禁止携带领域具体类型（使用字符串/Display）
//! 🟩 与「分领域错误原则」不冲突——领域内用 XxxError，跨层用此 Message

use bevy::prelude::*;

/// 游戏错误 Message——跨层统一错误上报通道
///
/// 使用方式：
/// ```ignore
/// fn my_system(mut commands: Commands) {
///     if let Err(e) = fallible_operation() {
///         commands.write_message(GameErrorMessage::new(
///             ErrorType::DomainError,
///             e.to_string(),
///             "battle",
///         ));
///     }
/// }
/// ```
#[derive(Message, Debug, Clone)]
pub struct GameErrorMessage {
    /// 错误类型分类（对应宪法 §13.9.2 失败分类学）
    pub error_type: ErrorType,
    /// 人类可读的错误描述
    pub message: String,
    /// 源模块名（如 "battle"、"buff"、"infrastructure"）
    pub source: &'static str,
    /// 结构化上下文键值对
    pub context: Vec<(&'static str, String)>,
}

/// 错误类型分类（对应宪法 §13.9.2 失败分类学）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// 规则失败：业务正常不满足，非程序错误
    /// 例：法力不足、超出攻击范围
    RuleFailure,
    /// 领域错误：领域内预期内的异常
    /// 例：技能配置不存在、BuffID 无效
    DomainError,
    /// 基础设施错误：底层能力异常
    /// 例：资源加载失败、存档 IO 错误
    Infrastructure,
    /// 程序 Bug：非法状态、逻辑断言失败
    /// 例：状态机非法跳转、数据一致性破坏
    Bug,
}

impl ErrorType {
    /// 获取简写标签（用于日志 target）
    pub fn label(&self) -> &'static str {
        match self {
            Self::RuleFailure => "RULE",
            Self::DomainError => "DOMAIN",
            Self::Infrastructure => "INFRA",
            Self::Bug => "BUG",
        }
    }
}

impl GameErrorMessage {
    /// 创建新的错误 Message
    pub fn new(error_type: ErrorType, message: impl Into<String>, source: &'static str) -> Self {
        Self {
            error_type,
            message: message.into(),
            source,
            context: Vec::new(),
        }
    }

    /// 附加结构化上下文
    pub fn with_context(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.context.push((key, value.into()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_type_label_对应分类() {
        assert_eq!(ErrorType::RuleFailure.label(), "RULE");
        assert_eq!(ErrorType::DomainError.label(), "DOMAIN");
        assert_eq!(ErrorType::Infrastructure.label(), "INFRA");
        assert_eq!(ErrorType::Bug.label(), "BUG");
    }

    #[test]
    fn game_error_message_可构建() {
        let msg = GameErrorMessage::new(ErrorType::DomainError, "技能配置不存在", "skill");
        assert_eq!(msg.source, "skill");
        assert!(msg.context.is_empty());
    }

    #[test]
    fn game_error_message_可附加上下文() {
        let msg = GameErrorMessage::new(ErrorType::Bug, "非法状态", "turn")
            .with_context("state", "PhaseConflict")
            .with_context("turn", "3");
        assert_eq!(msg.context.len(), 2);
        assert_eq!(msg.context[0], ("state", "PhaseConflict".to_string()));
    }

    #[test]
    fn game_error_message_是bevy_message() {
        fn assert_is_message<T: Message>() {}
        assert_is_message::<GameErrorMessage>();
    }
}
