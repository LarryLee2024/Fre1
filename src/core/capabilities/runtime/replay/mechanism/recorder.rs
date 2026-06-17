//! Replay Recorder Logic — 录制模式的核心逻辑
//!
//! 提供录制生命周期管理：开始帧、录制命令、结束帧、生成校验和。
//!
//! 详见 docs/04-data/infrastructure/replay_schema.md §3.6

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayError, ReplayFrame, ReplayHeader, ReplayLog, ReplayRecorder,
    ReplayValidator,
};

/// 录制流程执行器。
///
/// 封装单次录制会话的完整生命周期。
#[derive(Debug, Clone)]
pub struct RecordingSession {
    /// 录制器
    pub recorder: ReplayRecorder,
    /// 验证器
    pub validator: ReplayValidator,
    /// 回放头信息（录制完成后填充）
    pub header: Option<ReplayHeader>,
}

impl RecordingSession {
    /// 创建录制会话。
    pub fn new(checkpoint_interval: u32) -> Self {
        Self {
            recorder: ReplayRecorder::new(checkpoint_interval),
            validator: ReplayValidator::new(),
            header: None,
        }
    }

    /// 开始录制。
    pub fn start(&mut self, header: ReplayHeader, initial_seed_offset: u64) {
        self.header = Some(header);
        self.recorder.start_recording(initial_seed_offset);
        self.validator.start_recording();
    }

    /// 开始新的一帧。
    pub fn start_frame(&mut self, frame_number: u64, rng_seed_offset: u64) {
        self.recorder.start_frame(frame_number, rng_seed_offset);
    }

    /// 录制一个命令。
    pub fn record_command(&mut self, command: ReplayCommand) {
        self.recorder.record_command(command);
    }

    /// 结束当前帧（计算校验和）。
    pub fn finalize_frame(&mut self, checksum: u64) {
        self.recorder.set_frame_checksum(checksum);
        self.validator.record_checksum(checksum);
    }

    /// 停止录制并生成 ReplayLog。
    pub fn stop(&mut self, final_checksum: u64) -> Result<ReplayLog, ReplayError> {
        if !self.recorder.is_recording {
            return Err(ReplayError::NotRecording);
        }

        // 完成当前帧
        if let Some(frame) = self.recorder.current_frame.take() {
            self.recorder.frames.push(frame);
        }

        self.recorder.is_recording = false;
        self.validator.record_checksum(final_checksum);

        let mut header = self.header.take().ok_or(ReplayError::NotRecording)?;
        header.set_total_frames(self.recorder.frames.len() as u64);

        let mut log = ReplayLog::new(header);
        for frame in self.recorder.frames.drain(..) {
            log.add_frame(frame);
        }
        log.set_final_checksum(final_checksum);

        Ok(log)
    }

    /// 是否正在录制。
    pub fn is_recording(&self) -> bool {
        self.recorder.is_recording
    }

    /// 已录制的帧数。
    pub fn frame_count(&self) -> usize {
        self.recorder.frame_count()
    }
}

/// 简单的哈希计算（用于快速校验和）。
///
/// 注意：这不是加密安全的，仅用于回放一致性验证。
pub fn calculate_frame_checksum(frame: &ReplayFrame) -> u64 {
    let mut hash: u64 = frame.frame_number.wrapping_mul(0x9E37_79B9);

    for cmd in &frame.commands {
        let cmd_hash = match cmd {
            ReplayCommand::UnitMove { unit, path } => {
                let mut h = 0u64;
                for b in unit.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for p in path {
                    for b in p.bytes() {
                        h = h.wrapping_mul(31).wrapping_add(b as u64);
                    }
                }
                h
            }
            ReplayCommand::UseAbility {
                caster,
                ability_def_id,
                ..
            } => {
                let mut h = 0u64;
                for b in caster.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for b in ability_def_id.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h
            }
            ReplayCommand::UseItem {
                user,
                item_instance_id,
                ..
            } => {
                let mut h = 0u64;
                for b in user.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for b in item_instance_id.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h
            }
            ReplayCommand::SkipTurn { unit } => {
                let mut h = 0u64;
                for b in unit.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h
            }
            ReplayCommand::DialogueChoice { speaker, choice_id } => {
                let mut h = 0u64;
                for b in speaker.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for b in choice_id.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h
            }
            ReplayCommand::ReactionConfirm {
                reactor,
                trigger_def_id,
                accepted,
            } => {
                let mut h = if *accepted { 1u64 } else { 0u64 };
                for b in reactor.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for b in trigger_def_id.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h
            }
            ReplayCommand::ConfirmTargets {
                caster,
                ability_def_id,
                selected_targets,
            } => {
                let mut h = 0u64;
                for b in caster.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for b in ability_def_id.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for t in selected_targets {
                    for b in t.bytes() {
                        h = h.wrapping_mul(31).wrapping_add(b as u64);
                    }
                }
                h
            }
            ReplayCommand::Custom {
                command_type,
                params,
                ..
            } => {
                let mut h = 0u64;
                for b in command_type.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                for (k, v) in params {
                    for b in k.bytes() {
                        h = h.wrapping_mul(31).wrapping_add(b as u64);
                    }
                    for b in v.bytes() {
                        h = h.wrapping_mul(31).wrapping_add(b as u64);
                    }
                }
                h
            }
        };
        hash ^= cmd_hash;
    }

    hash
}

/// 验证帧号连续性。
pub fn validate_frame_sequence(frames: &[ReplayFrame]) -> Result<(), ReplayError> {
    for (i, frame) in frames.iter().enumerate() {
        if frame.frame_number != i as u64 {
            return Err(ReplayError::FrameNumberGap {
                expected: i as u64,
                got: frame.frame_number,
            });
        }
    }
    Ok(())
}

/// 验证版本兼容性。
pub fn validate_version(schema_version: u32, current_version: u32) -> Result<(), ReplayError> {
    if schema_version > current_version {
        return Err(ReplayError::VersionMismatch {
            expected: current_version,
            actual: schema_version,
        });
    }
    Ok(())
}
