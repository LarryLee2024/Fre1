//! Replay 值对象：录制器、播放器、验证器

use bevy::prelude::Reflect;

use super::types::{ReplayCommand, ReplayFrame, ReplayHeader};

/// 完整的回放日志。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ReplayLog {
    /// 头部元数据
    pub header: ReplayHeader,
    /// 命令帧序列
    pub frames: Vec<ReplayFrame>,
    /// 最终校验和（可选）
    pub final_checksum: Option<u64>,
}

impl ReplayLog {
    /// 创建回放日志。
    pub fn new(header: ReplayHeader) -> Self {
        Self {
            header,
            frames: Vec::new(),
            final_checksum: None,
        }
    }

    /// 添加帧。
    pub fn add_frame(&mut self, frame: ReplayFrame) {
        self.header.set_total_frames(self.frames.len() as u64 + 1);
        self.frames.push(frame);
    }

    /// 获取帧数量。
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// 设置最终校验和。
    pub fn set_final_checksum(&mut self, checksum: u64) {
        self.final_checksum = Some(checksum);
    }
}

/// 回放模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayMode {
    /// 完整回放（逐帧执行，检查 SyncCheckpoint）
    Full,
    /// 快速回放（跳过非关键帧，仅验证 Checkpoint）
    FastForward,
    /// 单步调试
    StepByStep,
}

impl ReplayMode {
    /// 返回模式名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Full => "Full",
            Self::FastForward => "FastForward",
            Self::StepByStep => "StepByStep",
        }
    }
}

/// 回放录制器——录制模式下记录命令和种子。
#[derive(Debug, Clone)]
pub struct ReplayRecorder {
    /// 是否正在录制
    pub is_recording: bool,
    /// 已完成的帧序列
    pub frames: Vec<ReplayFrame>,
    /// 当前正在录制的帧
    pub current_frame: Option<ReplayFrame>,
    /// 检查点间隔（每多少帧记录一次校验和）
    pub checkpoint_interval: u32,
    /// 帧内已录制的命令计数
    commands_in_frame: usize,
}

impl ReplayRecorder {
    /// 创建录制器。
    pub fn new(checkpoint_interval: u32) -> Self {
        Self {
            is_recording: false,
            frames: Vec::new(),
            current_frame: None,
            checkpoint_interval,
            commands_in_frame: 0,
        }
    }

    /// 开始录制。
    pub fn start_recording(&mut self, initial_seed_offset: u64) {
        self.is_recording = true;
        self.frames.clear();
        self.current_frame = Some(ReplayFrame::new(0, initial_seed_offset));
        self.commands_in_frame = 0;
    }

    /// 停止录制。
    pub fn stop_recording(&mut self) {
        self.is_recording = false;
        if let Some(frame) = self.current_frame.take() {
            self.frames.push(frame);
        }
    }

    /// 开始新的一帧。
    pub fn start_frame(&mut self, frame_number: u64, rng_seed_offset: u64) {
        if !self.is_recording {
            return;
        }

        // 完成当前帧
        if let Some(frame) = self.current_frame.take() {
            self.frames.push(frame);
        }

        self.current_frame = Some(ReplayFrame::new(frame_number, rng_seed_offset));
        self.commands_in_frame = 0;
    }

    /// 录制一个命令。
    pub fn record_command(&mut self, command: ReplayCommand) {
        if !self.is_recording {
            return;
        }

        if let Some(ref mut frame) = self.current_frame {
            frame.add_command(command);
            self.commands_in_frame += 1;
        }
    }

    /// 是否为检查点帧（需要记录校验和）。
    pub fn is_checkpoint_frame(&self, frame_number: u64) -> bool {
        self.checkpoint_interval > 0 && frame_number.is_multiple_of(self.checkpoint_interval as u64)
    }

    /// 设置当前帧的校验和。
    pub fn set_frame_checksum(&mut self, checksum: u64) {
        if let Some(ref mut frame) = self.current_frame {
            frame.set_checksum(checksum);
        }
    }

    /// 已录制的帧数。
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// 当前帧中的命令数。
    pub fn commands_in_current_frame(&self) -> usize {
        self.commands_in_frame
    }
}

/// 回放播放器——回放模式下逐帧执行命令。
#[derive(Debug, Clone)]
pub struct ReplayPlayer {
    /// 是否正在回放
    pub is_playing: bool,
    /// 待回放的帧序列
    pub frames: Vec<ReplayFrame>,
    /// 当前帧索引
    pub current_index: usize,
    /// 回放模式
    pub mode: ReplayMode,
    /// 当前帧的命令执行位置
    pub command_index: usize,
}

impl ReplayPlayer {
    /// 创建播放器。
    pub fn new(mode: ReplayMode) -> Self {
        Self {
            is_playing: false,
            frames: Vec::new(),
            current_index: 0,
            mode,
            command_index: 0,
        }
    }

    /// 加载回放日志。
    pub fn load(&mut self, log: &ReplayLog) {
        self.frames = log.frames.clone();
        self.current_index = 0;
        self.command_index = 0;
    }

    /// 开始回放。
    pub fn start_playing(&mut self) {
        self.is_playing = true;
        self.current_index = 0;
        self.command_index = 0;
    }

    /// 停止回放。
    pub fn stop_playing(&mut self) {
        self.is_playing = false;
    }

    /// 获取当前帧。
    pub fn current_frame(&self) -> Option<&ReplayFrame> {
        self.frames.get(self.current_index)
    }

    /// 推进到下一帧。
    ///
    /// 返回 false 表示已无更多帧（此时 current_index 被设到末尾）。
    pub fn advance_frame(&mut self) -> bool {
        if self.current_index + 1 < self.frames.len() {
            self.current_index += 1;
            self.command_index = 0;
            true
        } else {
            // 标记为已遍历完所有帧
            self.current_index = self.frames.len();
            false
        }
    }

    /// 获取当前帧的所有命令。
    pub fn current_commands(&self) -> Vec<&ReplayCommand> {
        self.frames
            .get(self.current_index)
            .map(|f| f.commands.iter().collect())
            .unwrap_or_default()
    }

    /// 是否已到末尾。
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.frames.len()
    }

    /// 总帧数。
    pub fn total_frames(&self) -> usize {
        self.frames.len()
    }

    /// 获取当前帧号。
    pub fn current_frame_number(&self) -> Option<u64> {
        self.frames.get(self.current_index).map(|f| f.frame_number)
    }
}

/// 回放验证器——录制时计算校验和，回放时比对。
#[derive(Debug, Clone)]
pub struct ReplayValidator {
    /// 是否录制模式
    pub recording: bool,
    /// 当前帧号
    pub current_frame: u64,
    /// 累计校验和
    pub accumulated_checksum: u64,
    /// 不一致记录
    pub mismatches: Vec<ReplayMismatch>,
}

/// 回放不一致记录。
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayMismatch {
    /// 不一致的帧号
    pub frame: u64,
    /// 预期的校验和
    pub expected_checksum: u64,
    /// 实际的校验和
    pub actual_checksum: u64,
}

impl ReplayValidator {
    /// 创建验证器。
    pub fn new() -> Self {
        Self {
            recording: false,
            current_frame: 0,
            accumulated_checksum: 0,
            mismatches: Vec::new(),
        }
    }

    /// 开始录制模式。
    pub fn start_recording(&mut self) {
        self.recording = true;
        self.current_frame = 0;
        self.accumulated_checksum = 0;
        self.mismatches.clear();
    }

    /// 开始回放验证模式。
    pub fn start_verification(&mut self) {
        self.recording = false;
        self.current_frame = 0;
        self.accumulated_checksum = 0;
        self.mismatches.clear();
    }

    /// 记录帧校验和（录制模式）。
    pub fn record_checksum(&mut self, checksum: u64) {
        self.accumulated_checksum ^= checksum;
        self.current_frame += 1;
    }

    /// 验证帧校验和（回放模式）。
    pub fn verify_checksum(&mut self, frame: u64, expected: u64, actual: u64) {
        self.accumulated_checksum ^= actual;
        if expected != actual {
            self.mismatches.push(ReplayMismatch {
                frame,
                expected_checksum: expected,
                actual_checksum: actual,
            });
        }
        self.current_frame += 1;
    }

    /// 是否存在不一致。
    pub fn has_mismatches(&self) -> bool {
        !self.mismatches.is_empty()
    }

    /// 不一致数量。
    pub fn mismatch_count(&self) -> usize {
        self.mismatches.len()
    }

    /// 获取累计校验和。
    pub fn accumulated_checksum(&self) -> u64 {
        self.accumulated_checksum
    }
}

impl Default for ReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 回放模式守卫——标记当前是否在回放模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct ReplayModeGuard {
    /// 是否处于回放模式
    pub is_replay: bool,
}

impl ReplayModeGuard {
    /// 创建正常模式守卫。
    pub fn normal() -> Self {
        Self { is_replay: false }
    }

    /// 创建回放模式守卫。
    pub fn replay_mode() -> Self {
        Self { is_replay: true }
    }
}

impl Default for ReplayModeGuard {
    fn default() -> Self {
        Self::normal()
    }
}
