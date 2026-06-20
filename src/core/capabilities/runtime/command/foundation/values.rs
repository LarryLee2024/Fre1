//! Command 值对象：命令队列与历史记录

use bevy::prelude::Resource;

use super::error::CommandError;
use super::types::{CommandSource, GameCommand, RecordedCommand};

/// 命令队列——统一命令入口。
///
/// 玩家、AI、Replay 都通过此 Resource 提交命令。
/// 在 PreUpdate 中执行 drain 并分发。
///
/// 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md §3
#[derive(Resource, Debug, Clone)]
pub struct CommandQueue {
    /// 待处理命令队列（当前帧）
    pending: Vec<GameCommand>,
    /// 历史记录（用于录制）
    history: Vec<RecordedCommand>,
    /// 最大队列大小（0 = 无限制）
    max_size: usize,
    /// 当前帧序号
    frame_number: u64,
}

impl CommandQueue {
    /// 创建空的命令队列。
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            history: Vec::new(),
            max_size: 0,
            frame_number: 0,
        }
    }

    /// 设置最大队列大小。
    pub fn set_max_size(&mut self, max: usize) {
        self.max_size = max;
    }

    /// 提交一个命令（不限来源）。
    pub fn push(&mut self, command: GameCommand) -> Result<(), CommandError> {
        if self.max_size > 0 && self.pending.len() >= self.max_size {
            return Err(CommandError::QueueFull(self.max_size));
        }
        self.pending.push(command);
        Ok(())
    }

    /// 提交并录制一个命令。
    pub fn push_recorded(
        &mut self,
        command: GameCommand,
        source: CommandSource,
    ) -> Result<(), CommandError> {
        self.push(command.clone())?;
        self.history
            .push(RecordedCommand::new(source, command, self.frame_number));
        Ok(())
    }

    /// 取出所有待处理命令供执行。
    pub fn drain(&mut self) -> Vec<GameCommand> {
        self.pending.drain(..).collect()
    }

    /// 推进帧序号。
    pub fn advance_frame(&mut self) {
        self.frame_number += 1;
    }

    /// 获取当前帧序号。
    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// 待处理命令数量。
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 历史记录数量。
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// 获取历史记录引用。
    pub fn history(&self) -> &[RecordedCommand] {
        &self.history
    }

    /// 清空历史记录。
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 是否有待处理命令。
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 命令历史——记录所有已执行的命令。
#[derive(Debug, Clone)]
pub struct CommandHistory {
    /// 已执行的命令记录
    entries: Vec<RecordedCommand>,
}

impl CommandHistory {
    /// 创建空的历史记录。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 记录一条已执行的命令。
    pub fn record(&mut self, entry: RecordedCommand) {
        self.entries.push(entry);
    }

    /// 获取所有记录。
    pub fn all(&self) -> &[RecordedCommand] {
        &self.entries
    }

    /// 记录数量。
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// 按来源筛选记录。
    pub fn filter_by_source(&self, source: CommandSource) -> Vec<&RecordedCommand> {
        self.entries.iter().filter(|e| e.source == source).collect()
    }

    /// 清空历史。
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
