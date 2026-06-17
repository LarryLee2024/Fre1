//! Replay Player Logic — 回放模式的核心逻辑
//!
//! 提供回放执行生命周期：加载、逐帧执行、校验验证。
//!
//! 详见 docs/04-data/infrastructure/replay_schema.md §3.6
//! 详见 docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md

use crate::core::capabilities::runtime::replay::foundation::{
    DeterministicRng, ReplayCommand, ReplayError, ReplayLog, ReplayMode, ReplayPlayer,
    ReplayValidator, RngSeeds,
};

use super::recorder::calculate_frame_checksum;

/// 回放执行器——管理完整的回放会话。
#[derive(Debug, Clone)]
pub struct PlaybackSession {
    /// 播放器
    pub player: ReplayPlayer,
    /// 确定性 RNG（回放模式下由种子驱动）
    pub rng: DeterministicRng,
    /// 验证器
    pub validator: ReplayValidator,
    /// 初始 RNG 种子
    initial_seed: u64,
}

impl PlaybackSession {
    /// 创建回放执行器。
    pub fn new(mode: ReplayMode, initial_seed: u64) -> Self {
        Self {
            player: ReplayPlayer::new(mode),
            rng: DeterministicRng::with_seed(initial_seed),
            validator: ReplayValidator::new(),
            initial_seed,
        }
    }

    /// 加载回放日志。
    pub fn load(&mut self, log: &ReplayLog) -> Result<(), ReplayError> {
        // 验证版本
        if log.header.schema_version > 1 {
            return Err(ReplayError::VersionMismatch {
                expected: 1,
                actual: log.header.schema_version,
            });
        }

        // 验证帧序列
        for (i, frame) in log.frames.iter().enumerate() {
            if frame.frame_number != i as u64 {
                return Err(ReplayError::FrameNumberGap {
                    expected: i as u64,
                    got: frame.frame_number,
                });
            }
        }

        if log.frames.is_empty() {
            return Err(ReplayError::EmptyLog);
        }

        // 设置初始种子
        self.initial_seed = log.header.initial_seed;
        let seeds = RngSeeds::uniform(log.header.initial_seed);
        self.rng.set_all_seeds(seeds);

        self.player.load(log);
        self.validator.start_verification();

        Ok(())
    }

    /// 开始回放。
    pub fn start(&mut self) {
        self.player.start_playing();
        // 设置第一帧的种子偏移
        if let Some(frame) = self.player.current_frame() {
            let seeds = RngSeeds::uniform(self.initial_seed.wrapping_add(frame.rng_seed_offset));
            self.rng.set_all_seeds(seeds);
        }
    }

    /// 获取当前帧的所有命令。
    pub fn current_commands(&self) -> Vec<&ReplayCommand> {
        self.player.current_commands()
    }

    /// 推进到下一帧（更新 RNG 种子）。
    pub fn advance_frame(&mut self) -> bool {
        if !self.player.advance_frame() {
            return false;
        }

        // 更新 RNG 种子为当前帧的偏移
        if let Some(frame) = self.player.current_frame() {
            let seeds = RngSeeds::uniform(self.initial_seed.wrapping_add(frame.rng_seed_offset));
            self.rng.set_all_seeds(seeds);
        }

        true
    }

    /// 验证当前帧的校验和。
    pub fn verify_current_frame(&mut self) -> Result<bool, ReplayError> {
        if let Some(frame) = self.player.current_frame() {
            let actual = calculate_frame_checksum(frame);
            if let Some(expected) = frame.checksum {
                self.validator
                    .verify_checksum(frame.frame_number, expected, actual);
                if expected != actual {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// 是否已完成回放。
    pub fn is_finished(&self) -> bool {
        self.player.is_finished()
    }

    /// 是否有校验不一致。
    pub fn has_mismatches(&self) -> bool {
        self.validator.has_mismatches()
    }

    /// 停止回放。
    pub fn stop(&mut self) {
        self.player.stop_playing();
    }

    /// 获取当前帧号。
    pub fn current_frame_number(&self) -> Option<u64> {
        self.player.current_frame_number()
    }

    /// 总帧数。
    pub fn total_frames(&self) -> usize {
        self.player.total_frames()
    }

    /// 获取 RNG 引用。
    pub fn rng(&self) -> &DeterministicRng {
        &self.rng
    }

    /// 获取 RNG 可变引用。
    pub fn rng_mut(&mut self) -> &mut DeterministicRng {
        &mut self.rng
    }
}

/// 快速回放——不逐帧验证，仅验证检查点。
pub fn fast_forward(session: &mut PlaybackSession) -> Result<(), ReplayError> {
    while !session.is_finished() {
        session.advance_frame();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::capabilities::runtime::replay::foundation::{ReplayFrame, RngStream};

    fn create_test_log() -> ReplayLog {
        let header = crate::core::capabilities::runtime::replay::foundation::ReplayHeader::new(
            1, "1.0", "test", 42,
        );

        let mut frame0 = ReplayFrame::new(0, 0);
        frame0.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

        let mut frame1 = ReplayFrame::new(1, 1);
        frame1.add_command(ReplayCommand::UnitMove {
            unit: "u1".into(),
            path: vec!["0,0".into()],
        });

        let mut log = ReplayLog::new(header);
        log.add_frame(frame0);
        log.add_frame(frame1);
        log
    }

    #[test]
    fn unit_060_playback_session_load() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        assert!(session.load(&log).is_ok());
    }

    #[test]
    fn unit_061_playback_session_empty_log() {
        let header = crate::core::capabilities::runtime::replay::foundation::ReplayHeader::new(
            1, "1.0", "test", 0,
        );
        let log = ReplayLog::new(header);
        let mut session = PlaybackSession::new(ReplayMode::Full, 0);
        assert_eq!(session.load(&log), Err(ReplayError::EmptyLog));
    }

    #[test]
    fn unit_062_playback_session_start_and_advance() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        session.load(&log).unwrap();
        session.start();

        assert_eq!(session.current_frame_number(), Some(0));
        assert!(session.advance_frame());
        assert_eq!(session.current_frame_number(), Some(1));
        assert!(!session.advance_frame()); // no more frames
        assert!(session.is_finished());
    }

    #[test]
    fn unit_063_playback_session_current_commands() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        session.load(&log).unwrap();
        session.start();

        let cmds = session.current_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].type_name(), "SkipTurn");
    }

    #[test]
    fn unit_064_playback_session_rng_determinism() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        session.load(&log).unwrap();
        session.start();

        let val_a = session.rng_mut().next_u64(RngStream::Combat);

        // Create another session with same seed
        let log2 = create_test_log();
        let mut session2 = PlaybackSession::new(ReplayMode::Full, 42);
        session2.load(&log2).unwrap();
        session2.start();
        let val_b = session2.rng_mut().next_u64(RngStream::Combat);

        assert_eq!(val_a, val_b);
    }

    #[test]
    fn unit_065_playback_verify_checksum() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        session.load(&log).unwrap();
        session.start();

        // Frame has no checksum set, so verification should pass
        assert!(session.verify_current_frame().is_ok());
    }

    #[test]
    fn unit_066_version_mismatch_rejected() {
        let header = crate::core::capabilities::runtime::replay::foundation::ReplayHeader::new(
            2, "2.0", "test", 0,
        );
        let mut frame = ReplayFrame::new(0, 0);
        frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });
        let mut log = ReplayLog::new(header);
        log.add_frame(frame);

        let mut session = PlaybackSession::new(ReplayMode::Full, 0);
        assert!(session.load(&log).is_err());
    }

    #[test]
    fn unit_067_fast_forward() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::FastForward, 42);
        session.load(&log).unwrap();
        session.start();

        fast_forward(&mut session).unwrap();
        assert!(session.is_finished());
    }

    #[test]
    fn unit_068_playback_session_stop() {
        let log = create_test_log();
        let mut session = PlaybackSession::new(ReplayMode::Full, 42);
        session.load(&log).unwrap();
        session.start();
        assert!(session.player.is_playing);

        session.stop();
        assert!(!session.player.is_playing);
    }
}
