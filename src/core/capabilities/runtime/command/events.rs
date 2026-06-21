//! Command 领域事件
//!
//! 定义命令提交和执行过程中的核心事件。

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::{CommandSource, GameCommand};

/// 命令提交时触发。
///
/// 订阅者：命令队列（入队处理）、日志。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandSubmitted {
    /// 提交的命令
    pub command: GameCommand,
    /// 命令来源
    pub source: CommandSource,
    /// 帧序号
    pub frame_number: u64,
}

/// 命令执行成功时触发。
///
/// 订阅者：录制系统（记录到回放日志）、日志。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandExecuted {
    /// 已执行的命令
    pub command: GameCommand,
    /// 命令来源
    pub source: CommandSource,
}

/// 命令被拒绝时触发。
///
/// 订阅者：UI（提示玩家命令无效）、日志。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandRejected {
    /// 被拒绝的命令
    pub command: GameCommand,
    /// 来源
    pub source: CommandSource,
    /// 拒绝原因
    pub reason: String,
}
