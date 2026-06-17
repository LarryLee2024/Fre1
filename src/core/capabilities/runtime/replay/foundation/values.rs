//! Replay 值对象：录制器、播放器、确定性 RNG、验证器

use std::collections::HashMap;

use super::types::{ReplayCommand, ReplayFrame, ReplayHeader, RngSeeds, RngStream};

/// 完整的回放日志。
#[derive(Debug, Clone, PartialEq)]
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

/// 确定性 RNG——每流一个独立实例。
///
/// 所有业务随机操作通过此资源进行，确保回放确定性。
///
/// 详见 ADR-041 §3
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    /// 各流当前种子
    seeds: RngSeeds,
    /// 各流调用计数器（用于产生不同值）
    counters: HashMap<RngStream, u64>,
}

impl DeterministicRng {
    /// 创建确定性 RNG。
    pub fn new(seeds: RngSeeds) -> Self {
        let mut counters = HashMap::new();
        for stream in &RngStream::all() {
            counters.insert(*stream, 0);
        }
        Self { seeds, counters }
    }

    /// 使用统一初始种子创建。
    pub fn with_seed(seed: u64) -> Self {
        Self::new(RngSeeds::uniform(seed))
    }

    /// 获取指定流的种子。
    pub fn get_seed(&self, stream: RngStream) -> u64 {
        self.seeds.get(stream)
    }

    /// 设置指定流的种子。
    pub fn set_seed(&mut self, stream: RngStream, seed: u64) {
        self.seeds.set(stream, seed);
        if let Some(counter) = self.counters.get_mut(&stream) {
            *counter = 0;
        }
    }

    /// 获取所有流的种子。
    pub fn get_all_seeds(&self) -> RngSeeds {
        self.seeds
    }

    /// 同步设置所有流种子（回放模式）。
    pub fn set_all_seeds(&mut self, seeds: RngSeeds) {
        self.seeds = seeds;
        for counter in self.counters.values_mut() {
            *counter = 0;
        }
    }

    /// 生成指定流的下一个伪随机数（0..u64::MAX）。
    pub fn next_u64(&mut self, stream: RngStream) -> u64 {
        let counter = self.counters.get(&stream).copied().unwrap_or(0);
        self.counters.insert(stream, counter + 1);

        // 简单的确定性哈希：种子 + 流偏置 + 计数器
        let stream_offset = match stream {
            RngStream::Combat => 0x9E37_79B9_7F4A_7C15u64,
            RngStream::Drop => 0xBF58_4766_71CE_4E5Bu64,
            RngStream::AI => 0x3C6E_F372_FE94_7A9Bu64,
            RngStream::World => 0x6A09_E667_F3BC_C4C9u64,
        };

        let state = self
            .seeds
            .get(stream)
            .wrapping_add(stream_offset)
            .wrapping_add(counter);

        // 混合：先乘法扩散低位，再 xorshift 扩散高位
        // 使用 MurmurHash3 风格的乘法混合器，解决 xorshift 在相邻输入下高位不扩散的问题
        let mut x = state;
        x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        x ^= x >> 27;
        x = x.wrapping_mul(0xBF58_4766_71CE_4E5B);
        x ^= x >> 31;
        x
    }

    /// 生成指定流的 f32 伪随机数（0.0..1.0）。
    pub fn next_f32(&mut self, stream: RngStream) -> f32 {
        let val = self.next_u64(stream);
        (val >> 11) as f32 * (1.0 / (1u64 << 53) as f32)
    }

    /// 生成指定流的 bool 伪随机数（给定概率）。
    pub fn gen_bool(&mut self, stream: RngStream, probability: f32) -> bool {
        self.next_f32(stream) < probability
    }

    /// 生成指定流在 [min, max) 范围内的整数。
    pub fn gen_range(&mut self, stream: RngStream, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        let range = max - min;
        min + (self.next_u64(stream) % range)
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
        self.checkpoint_interval > 0 && frame_number % self.checkpoint_interval as u64 == 0
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── DeterministicRng ─────────────────────────────

    #[test]
    fn unit_020_deterministic_rng_uniform_seed() {
        let mut rng = DeterministicRng::with_seed(42);
        let val1 = rng.next_u64(RngStream::Combat);
        let val2 = rng.next_u64(RngStream::Combat);
        // Same seed, same first call → should be deterministic
        let mut rng2 = DeterministicRng::with_seed(42);
        assert_eq!(val1, rng2.next_u64(RngStream::Combat));
        // Second call should differ from first
        assert_ne!(val1, val2);
    }

    #[test]
    fn unit_021_deterministic_rng_different_streams() {
        let mut rng = DeterministicRng::with_seed(100);
        let combat = rng.next_u64(RngStream::Combat);
        let drop = rng.next_u64(RngStream::Drop);
        // Different streams with same seed → different values due to stream offset
        assert_ne!(combat, drop);
    }

    #[test]
    fn unit_022_deterministic_rng_f32_range() {
        let mut rng = DeterministicRng::with_seed(42);
        for _ in 0..100 {
            let val = rng.next_f32(RngStream::Combat);
            assert!(val >= 0.0);
            assert!(val < 1.0);
        }
    }

    #[test]
    fn unit_023_deterministic_rng_gen_bool() {
        let mut rng = DeterministicRng::with_seed(42);
        let count_true = (0..1000)
            .filter(|_| rng.gen_bool(RngStream::Combat, 0.5))
            .count();
        // With 50% probability, roughly half should be true
        assert!(count_true > 300 && count_true < 700);
    }

    #[test]
    fn unit_024_deterministic_rng_gen_range() {
        let mut rng = DeterministicRng::with_seed(42);
        for _ in 0..100 {
            let val = rng.gen_range(RngStream::Combat, 5, 10);
            assert!(val >= 5);
            assert!(val < 10);
        }
    }

    #[test]
    fn unit_025_deterministic_rng_set_seed_resets() {
        let mut rng = DeterministicRng::with_seed(42);
        let first = rng.next_u64(RngStream::Combat);

        rng.set_seed(RngStream::Combat, 42);
        let after_reset = rng.next_u64(RngStream::Combat);
        assert_eq!(first, after_reset);
    }

    #[test]
    fn unit_026_deterministic_rng_set_all_seeds() {
        let mut rng = DeterministicRng::with_seed(1);
        let seeds = RngSeeds::new(10, 20, 30, 40);
        rng.set_all_seeds(seeds);
        assert_eq!(rng.get_seed(RngStream::Combat), 10);
        assert_eq!(rng.get_seed(RngStream::Drop), 20);
    }

    // ── ReplayRecorder ───────────────────────────────

    #[test]
    fn unit_027_recorder_initial_state() {
        let recorder = ReplayRecorder::new(60);
        assert!(!recorder.is_recording);
        assert_eq!(recorder.frame_count(), 0);
    }

    #[test]
    fn unit_028_recorder_start_stop() {
        let mut recorder = ReplayRecorder::new(60);
        recorder.start_recording(0);
        assert!(recorder.is_recording);

        recorder.stop_recording();
        assert!(!recorder.is_recording);
    }

    #[test]
    fn unit_029_recorder_record_command() {
        let mut recorder = ReplayRecorder::new(60);
        recorder.start_recording(0);

        recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
        assert_eq!(recorder.commands_in_current_frame(), 1);
    }

    #[test]
    fn unit_030_recorder_frame_boundary() {
        let mut recorder = ReplayRecorder::new(60);
        recorder.start_recording(100);

        recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
        recorder.start_frame(1, 101);
        recorder.record_command(ReplayCommand::SkipTurn { unit: "u2".into() });

        recorder.stop_recording();
        assert_eq!(recorder.frame_count(), 2);
    }

    #[test]
    fn unit_031_recorder_checkpoint_frame() {
        let recorder = ReplayRecorder::new(10);
        assert!(recorder.is_checkpoint_frame(0));
        assert!(!recorder.is_checkpoint_frame(1));
        assert!(recorder.is_checkpoint_frame(10));
    }

    #[test]
    fn unit_032_recorder_not_recording_ignores() {
        let mut recorder = ReplayRecorder::new(60);
        recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
        // Not recording → commands should be ignored
        assert_eq!(recorder.commands_in_current_frame(), 0);
    }

    // ── ReplayPlayer ─────────────────────────────────

    #[test]
    fn unit_033_player_initial_state() {
        let player = ReplayPlayer::new(ReplayMode::Full);
        assert!(!player.is_playing);
        assert!(player.is_finished());
    }

    #[test]
    fn unit_034_player_load_and_play() {
        let log = create_test_replay_log();
        let mut player = ReplayPlayer::new(ReplayMode::Full);

        player.load(&log);
        player.start_playing();
        assert!(player.is_playing);
        assert!(!player.is_finished());
    }

    #[test]
    fn unit_035_player_advance_frame() {
        let log = create_test_replay_log();
        let mut player = ReplayPlayer::new(ReplayMode::Full);

        player.load(&log);
        player.start_playing();
        assert!(player.advance_frame());
        assert_eq!(player.current_frame_number(), Some(1));
    }

    #[test]
    fn unit_036_player_current_commands() {
        let log = create_test_replay_log();
        let mut player = ReplayPlayer::new(ReplayMode::Full);

        player.load(&log);
        player.start_playing();

        let cmds = player.current_commands();
        assert!(!cmds.is_empty());
        assert_eq!(cmds[0].type_name(), "SkipTurn");
    }

    #[test]
    fn unit_037_player_is_finished() {
        let log = create_test_replay_log();
        let mut player = ReplayPlayer::new(ReplayMode::Full);

        player.load(&log);
        player.start_playing();
        player.advance_frame();
        // After advancing past the last frame
        assert!(!player.advance_frame()); // no more frames
    }

    #[test]
    fn unit_038_replay_mode_name() {
        assert_eq!(ReplayMode::Full.name(), "Full");
        assert_eq!(ReplayMode::FastForward.name(), "FastForward");
        assert_eq!(ReplayMode::StepByStep.name(), "StepByStep");
    }

    // ── ReplayValidator ──────────────────────────────

    #[test]
    fn unit_039_validator_initial() {
        let v = ReplayValidator::new();
        assert_eq!(v.current_frame, 0);
        assert!(!v.has_mismatches());
    }

    #[test]
    fn unit_040_validator_no_mismatch() {
        let mut v = ReplayValidator::new();
        v.start_verification();
        v.verify_checksum(0, 0xABCD, 0xABCD);
        assert!(!v.has_mismatches());
    }

    #[test]
    fn unit_041_validator_mismatch_detected() {
        let mut v = ReplayValidator::new();
        v.start_verification();
        v.verify_checksum(0, 0xABCD, 0xDEAD);
        assert!(v.has_mismatches());
        assert_eq!(v.mismatch_count(), 1);
    }

    #[test]
    fn unit_042_validator_accumulated_checksum() {
        let mut v = ReplayValidator::new();
        v.start_recording();
        v.record_checksum(0xAAAA);
        v.record_checksum(0xBBBB);
        // XOR of both
        assert_eq!(v.accumulated_checksum(), 0xAAAA ^ 0xBBBB);
    }

    #[test]
    fn unit_043_replay_mode_guard() {
        let guard = ReplayModeGuard::normal();
        assert!(!guard.is_replay);

        let guard = ReplayModeGuard::replay_mode();
        assert!(guard.is_replay);
    }

    // ── ReplayLog ────────────────────────────────────

    #[test]
    fn unit_044_replay_log_creation() {
        let header = ReplayHeader::new(1, "1.0", "scene_001", 42);
        let log = ReplayLog::new(header);
        assert_eq!(log.frame_count(), 0);
    }

    #[test]
    fn unit_045_replay_log_add_frame() {
        let header = ReplayHeader::new(1, "1.0", "scene_001", 42);
        let mut log = ReplayLog::new(header);
        log.add_frame(ReplayFrame::new(0, 100));
        log.add_frame(ReplayFrame::new(1, 101));
        assert_eq!(log.frame_count(), 2);
    }

    #[test]
    fn unit_046_replay_log_final_checksum() {
        let header = ReplayHeader::new(1, "1.0", "scene", 0);
        let mut log = ReplayLog::new(header);
        log.set_final_checksum(0x1234);
        assert_eq!(log.final_checksum, Some(0x1234));
    }

    // ── Helpers ──────────────────────────────────────

    fn create_test_replay_log() -> ReplayLog {
        let header = ReplayHeader::new(1, "1.0", "test_scene", 42);

        let mut frame0 = ReplayFrame::new(0, 100);
        frame0.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

        let mut frame1 = ReplayFrame::new(1, 101);
        frame1.add_command(ReplayCommand::UnitMove {
            unit: "u1".into(),
            path: vec!["0,0".into(), "1,0".into()],
        });

        let mut log = ReplayLog::new(header);
        log.add_frame(frame0);
        log.add_frame(frame1);
        log
    }
}
