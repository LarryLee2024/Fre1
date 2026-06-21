//! Command 领域事件

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::{CommandSource, GameCommand};

/// 命令提交时触发。
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
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandExecuted {
    /// 已执行的命令
    pub command: GameCommand,
    /// 命令来源
    pub source: CommandSource,
}

/// 命令被拒绝时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandRejected {
    /// 被拒绝的命令
    pub command: GameCommand,
    /// 来源
    pub source: CommandSource,
    /// 拒绝原因
    pub reason: String,
}
