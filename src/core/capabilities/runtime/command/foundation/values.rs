//! Command 值对象：命令队列与历史记录

use super::types::{CommandError, CommandSource, GameCommand, RecordedCommand};

/// 命令队列——统一命令入口。
///
/// 玩家、AI、Replay 都通过此 Resource 提交命令。
/// 在 PreUpdate 中执行 drain 并分发。
///
/// 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md §3
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_move_cmd() -> GameCommand {
        GameCommand::MoveUnit {
            unit_id: "unit_001".into(),
            path: vec!["0,0".into()],
        }
    }

    #[test]
    fn unit_020_queue_empty() {
        let queue = CommandQueue::new();
        assert_eq!(queue.pending_count(), 0);
        assert!(!queue.has_pending());
    }

    #[test]
    fn unit_021_queue_push() {
        let mut queue = CommandQueue::new();
        assert!(queue.push(make_move_cmd()).is_ok());
        assert_eq!(queue.pending_count(), 1);
        assert!(queue.has_pending());
    }

    #[test]
    fn unit_022_queue_drain() {
        let mut queue = CommandQueue::new();
        queue.push(make_move_cmd()).unwrap();
        queue.push(GameCommand::OpenMenu).unwrap();

        let drained = queue.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn unit_023_queue_max_size() {
        let mut queue = CommandQueue::new();
        queue.set_max_size(2);
        assert!(queue.push(make_move_cmd()).is_ok());
        assert!(queue.push(make_move_cmd()).is_ok());
        assert_eq!(queue.push(make_move_cmd()), Err(CommandError::QueueFull(2)));
    }

    #[test]
    fn unit_024_queue_push_recorded() {
        let mut queue = CommandQueue::new();
        assert!(
            queue
                .push_recorded(make_move_cmd(), CommandSource::Player)
                .is_ok()
        );
        assert_eq!(queue.pending_count(), 1);
        assert_eq!(queue.history_count(), 1);
    }

    #[test]
    fn unit_025_queue_advance_frame() {
        let mut queue = CommandQueue::new();
        assert_eq!(queue.frame_number(), 0);
        queue.advance_frame();
        assert_eq!(queue.frame_number(), 1);
    }

    #[test]
    fn unit_026_queue_history() {
        let mut queue = CommandQueue::new();
        queue
            .push_recorded(make_move_cmd(), CommandSource::Player)
            .unwrap();

        let history = queue.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].source, CommandSource::Player);
    }

    #[test]
    fn unit_027_command_history_new() {
        let hist = CommandHistory::new();
        assert_eq!(hist.count(), 0);
    }

    #[test]
    fn unit_028_command_history_record() {
        let mut hist = CommandHistory::new();
        let cmd = GameCommand::OpenMenu;
        hist.record(RecordedCommand::new(CommandSource::System, cmd, 1));
        assert_eq!(hist.count(), 1);
    }

    #[test]
    fn unit_029_command_history_filter() {
        let mut hist = CommandHistory::new();
        hist.record(RecordedCommand::new(
            CommandSource::Player,
            make_move_cmd(),
            1,
        ));
        hist.record(RecordedCommand::new(CommandSource::AI, make_move_cmd(), 2));

        let player_cmds = hist.filter_by_source(CommandSource::Player);
        assert_eq!(player_cmds.len(), 1);

        let replay_cmds = hist.filter_by_source(CommandSource::Replay);
        assert_eq!(replay_cmds.len(), 0);
    }

    #[test]
    fn unit_030_queue_drain_frame_advance() {
        let mut queue = CommandQueue::new();

        // Frame 1
        queue.push(make_move_cmd()).unwrap();
        let _f1_cmds = queue.drain();
        queue.advance_frame();

        // Frame 2
        queue
            .push(GameCommand::EndTurn {
                unit_id: "unit_001".into(),
            })
            .unwrap();
        let f2_cmds = queue.drain();
        assert_eq!(f2_cmds.len(), 1);
        assert_eq!(f2_cmds[0].name(), "EndTurn");
    }
}
