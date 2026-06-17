//! Replay 领域事件

use bevy::prelude::*;

/// 回放开始时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReplayStarted {
    /// 回放场景标识
    pub scene_id: String,
    /// 总帧数
    pub total_frames: u64,
}

/// 每帧处理完成时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReplayFrameProcessed {
    /// 当前帧序号
    pub frame_number: u64,
    /// 总帧数
    pub total_frames: u64,
    /// 本帧命令数
    pub commands_in_frame: usize,
}

/// 回放完成时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReplayCompleted {
    /// 总帧数
    pub total_frames: u64,
    /// 总命令数
    pub total_commands: u64,
    /// 是否有校验不一致
    pub has_mismatches: bool,
}

/// 录制开始时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct RecordingStarted {
    /// 场景标识
    pub scene_id: String,
    /// 初始种子
    pub initial_seed: u64,
}

/// 录制完成时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct RecordingCompleted {
    /// 录制的帧数
    pub frames_recorded: u64,
    /// 录制的命令数
    pub commands_recorded: u64,
}

/// 校验不一致检测到时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ReplayMismatchDetected {
    /// 帧号
    pub frame: u64,
    /// 预期的校验和
    pub expected_checksum: u64,
    /// 实际的校验和
    pub actual_checksum: u64,
}
