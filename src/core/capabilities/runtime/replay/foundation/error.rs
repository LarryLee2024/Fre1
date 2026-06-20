//! Replay 领域错误枚举。
//!
//! 定义回放操作过程中可能出现的错误类型。

use bevy::prelude::Reflect;

/// 回放领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error, Reflect)]
pub enum ReplayError {
    /// 版本不兼容
    #[error("replay version mismatch: expected v{expected}, got v{actual}")]
    VersionMismatch {
        /// 期望版本
        expected: u32,
        /// 实际版本
        actual: u32,
    },
    /// 帧序号不连续
    #[error("frame number gap: expected {expected}, got {got}")]
    FrameNumberGap {
        /// 期望帧号
        expected: u64,
        /// 实际帧号
        got: u64,
    },
    /// 校验和不匹配
    #[error("checksum mismatch at frame {frame}: expected {expected:x}, got {actual:x}")]
    ChecksumMismatch {
        /// 帧序号
        frame: u64,
        /// 期望校验和
        expected: u64,
        /// 实际校验和
        actual: u64,
    },
    /// 未在录制模式
    #[error("not in recording mode")]
    NotRecording,
    /// 未在回放模式
    #[error("not in playback mode")]
    NotPlaying,
    /// 回放日志为空
    #[error("replay log is empty")]
    EmptyLog,
}
