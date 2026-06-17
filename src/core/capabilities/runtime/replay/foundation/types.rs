//! Replay 基础类型与枚举
//!
//! 定义回放帧、录制命令、RNG 流以及领域错误。
//!
//! 详见 docs/04-data/infrastructure/replay_schema.md

/// 回放帧——单帧的命令集合 + 种子信息。
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayFrame {
    /// 帧序号（从 0 开始）
    pub frame_number: u64,
    /// 本帧的所有命令
    pub commands: Vec<ReplayCommand>,
    /// 本帧的 RNG 种子偏移
    pub rng_seed_offset: u64,
    /// 校验和（可选）
    pub checksum: Option<u64>,
}

impl ReplayFrame {
    /// 创建新的回放帧。
    pub fn new(frame_number: u64, rng_seed_offset: u64) -> Self {
        Self {
            frame_number,
            commands: Vec::new(),
            rng_seed_offset,
            checksum: None,
        }
    }

    /// 添加一个命令到帧中。
    pub fn add_command(&mut self, command: ReplayCommand) {
        self.commands.push(command);
    }

    /// 设置校验和。
    pub fn set_checksum(&mut self, checksum: u64) {
        self.checksum = Some(checksum);
    }

    /// 命令数量。
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// 原子命令——回放的最小可录制单元。
///
/// 详见 replay_schema.md §3.4
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayCommand {
    /// 单位移动
    UnitMove {
        /// 单位标识
        unit: String,
        /// 移动路径
        path: Vec<String>,
    },
    /// 技能使用
    UseAbility {
        /// 施法者标识
        caster: String,
        /// 能力 Def ID
        ability_def_id: String,
        /// 目标
        target: AbilityTarget,
    },
    /// 物品使用
    UseItem {
        /// 使用者标识
        user: String,
        /// 物品实例 ID
        item_instance_id: String,
        /// 目标（可选）
        target: Option<String>,
    },
    /// 跳过回合
    SkipTurn {
        /// 当前单位标识
        unit: String,
    },
    /// 对话选择
    DialogueChoice {
        /// 对话者标识
        speaker: String,
        /// 选项 ID
        choice_id: String,
    },
    /// 反应触发确认
    ReactionConfirm {
        /// 反应者标识
        reactor: String,
        /// 触发 Def ID
        trigger_def_id: String,
        /// 是否接受
        accepted: bool,
    },
    /// 目标选择确认
    ConfirmTargets {
        /// 施法者标识
        caster: String,
        /// 能力 Def ID
        ability_def_id: String,
        /// 已选择的目标
        selected_targets: Vec<String>,
    },
    /// 自定义命令（由 Domain 扩展）
    Custom {
        /// 领域名称
        domain: String,
        /// 命令类型
        command_type: String,
        /// 参数
        params: Vec<(String, String)>,
    },
}

impl ReplayCommand {
    /// 返回命令的类型名称。
    pub fn type_name(&self) -> &str {
        match self {
            Self::UnitMove { .. } => "UnitMove",
            Self::UseAbility { .. } => "UseAbility",
            Self::UseItem { .. } => "UseItem",
            Self::SkipTurn { .. } => "SkipTurn",
            Self::DialogueChoice { .. } => "DialogueChoice",
            Self::ReactionConfirm { .. } => "ReactionConfirm",
            Self::ConfirmTargets { .. } => "ConfirmTargets",
            Self::Custom { .. } => "Custom",
        }
    }
}

/// 技能目标类型。
#[derive(Debug, Clone, PartialEq)]
pub enum AbilityTarget {
    /// 单体目标
    Single(String),
    /// 区域目标（位置坐标）
    Area(String),
    /// 无目标（如自我施法）
    None,
}

/// RNG 流——用途独立的随机数流。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RngStream {
    /// 战斗（命中/暴击/伤害浮动）
    Combat,
    /// 掉落/制造随机
    Drop,
    /// AI 决策随机
    AI,
    /// 世界事件随机
    World,
}

impl RngStream {
    /// 返回流名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Combat => "Combat",
            Self::Drop => "Drop",
            Self::AI => "AI",
            Self::World => "World",
        }
    }

    /// 所有流的列表。
    pub fn all() -> [Self; 4] {
        [Self::Combat, Self::Drop, Self::AI, Self::World]
    }
}

/// RNG 种子集合——每个流独立种子。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RngSeeds {
    /// 战斗种子
    pub combat_seed: u64,
    /// 掉落种子
    pub drop_seed: u64,
    /// AI 种子
    pub ai_seed: u64,
    /// 世界种子
    pub world_seed: u64,
}

impl RngSeeds {
    /// 创建统一的种子集合（所有流使用同一种子）。
    pub fn uniform(seed: u64) -> Self {
        Self {
            combat_seed: seed,
            drop_seed: seed,
            ai_seed: seed,
            world_seed: seed,
        }
    }

    /// 创建独立种子的集合。
    pub fn new(combat: u64, drop: u64, ai: u64, world: u64) -> Self {
        Self {
            combat_seed: combat,
            drop_seed: drop,
            ai_seed: ai,
            world_seed: world,
        }
    }

    /// 获取指定流的种子。
    pub fn get(&self, stream: RngStream) -> u64 {
        match stream {
            RngStream::Combat => self.combat_seed,
            RngStream::Drop => self.drop_seed,
            RngStream::AI => self.ai_seed,
            RngStream::World => self.world_seed,
        }
    }

    /// 设置指定流的种子。
    pub fn set(&mut self, stream: RngStream, seed: u64) {
        match stream {
            RngStream::Combat => self.combat_seed = seed,
            RngStream::Drop => self.drop_seed = seed,
            RngStream::AI => self.ai_seed = seed,
            RngStream::World => self.world_seed = seed,
        }
    }
}

/// 回放头信息——回放日志元数据。
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayHeader {
    /// Schema 版本
    pub schema_version: u32,
    /// 录制的游戏版本
    pub game_version: String,
    /// 场景标识
    pub scene_id: String,
    /// 参与实体列表
    pub participants: Vec<String>,
    /// 初始种子
    pub initial_seed: u64,
    /// 总帧数
    pub total_frames: u64,
}

impl ReplayHeader {
    /// 创建回放头信息。
    pub fn new(
        schema_version: u32,
        game_version: impl Into<String>,
        scene_id: impl Into<String>,
        initial_seed: u64,
    ) -> Self {
        Self {
            schema_version,
            game_version: game_version.into(),
            scene_id: scene_id.into(),
            participants: Vec::new(),
            initial_seed,
            total_frames: 0,
        }
    }

    /// 添加参与者。
    pub fn add_participant(&mut self, entity_id: impl Into<String>) {
        self.participants.push(entity_id.into());
    }

    /// 设置总帧数。
    pub fn set_total_frames(&mut self, total: u64) {
        self.total_frames = total;
    }
}

/// 回放领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayError {
    /// 版本不兼容
    VersionMismatch { expected: u32, actual: u32 },
    /// 帧序号不连续
    FrameNumberGap { expected: u64, got: u64 },
    /// 校验和不匹配
    ChecksumMismatch {
        frame: u64,
        expected: u64,
        actual: u64,
    },
    /// 未在录制模式
    NotRecording,
    /// 未在回放模式
    NotPlaying,
    /// 回放日志为空
    EmptyLog,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "replay version mismatch: expected v{}, got v{}",
                    expected, actual
                )
            }
            Self::FrameNumberGap { expected, got } => {
                write!(f, "frame number gap: expected {}, got {}", expected, got)
            }
            Self::ChecksumMismatch {
                frame,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "checksum mismatch at frame {}: expected {:x}, got {:x}",
                    frame, expected, actual
                )
            }
            Self::NotRecording => write!(f, "not in recording mode"),
            Self::NotPlaying => write!(f, "not in playback mode"),
            Self::EmptyLog => write!(f, "replay log is empty"),
        }
    }
}

impl std::error::Error for ReplayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_replay_frame_creation() {
        let frame = ReplayFrame::new(0, 100);
        assert_eq!(frame.frame_number, 0);
        assert_eq!(frame.rng_seed_offset, 100);
        assert!(frame.commands.is_empty());
    }

    #[test]
    fn unit_002_replay_frame_add_command() {
        let mut frame = ReplayFrame::new(0, 0);
        frame.add_command(ReplayCommand::SkipTurn {
            unit: "unit_001".into(),
        });
        assert_eq!(frame.command_count(), 1);
    }

    #[test]
    fn unit_003_replay_frame_set_checksum() {
        let mut frame = ReplayFrame::new(0, 0);
        frame.set_checksum(0xDEADBEEF);
        assert_eq!(frame.checksum, Some(0xDEADBEEF));
    }

    #[test]
    fn unit_004_replay_command_type_name() {
        let cmd = ReplayCommand::UnitMove {
            unit: "u1".into(),
            path: vec![],
        };
        assert_eq!(cmd.type_name(), "UnitMove");

        let cmd = ReplayCommand::Custom {
            domain: "test".into(),
            command_type: "ping".into(),
            params: vec![],
        };
        assert_eq!(cmd.type_name(), "Custom");
    }

    #[test]
    fn unit_005_rng_stream_name() {
        assert_eq!(RngStream::Combat.name(), "Combat");
        assert_eq!(RngStream::Drop.name(), "Drop");
        assert_eq!(RngStream::AI.name(), "AI");
        assert_eq!(RngStream::World.name(), "World");
    }

    #[test]
    fn unit_006_rng_stream_all() {
        let all = RngStream::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn unit_007_rng_seeds_uniform() {
        let seeds = RngSeeds::uniform(42);
        assert_eq!(seeds.combat_seed, 42);
        assert_eq!(seeds.drop_seed, 42);
        assert_eq!(seeds.ai_seed, 42);
        assert_eq!(seeds.world_seed, 42);
    }

    #[test]
    fn unit_008_rng_seeds_independent() {
        let seeds = RngSeeds::new(1, 2, 3, 4);
        assert_eq!(seeds.get(RngStream::Combat), 1);
        assert_eq!(seeds.get(RngStream::Drop), 2);
        assert_eq!(seeds.get(RngStream::AI), 3);
        assert_eq!(seeds.get(RngStream::World), 4);
    }

    #[test]
    fn unit_009_rng_seeds_set() {
        let mut seeds = RngSeeds::uniform(0);
        seeds.set(RngStream::Combat, 100);
        assert_eq!(seeds.combat_seed, 100);
    }

    #[test]
    fn unit_010_replay_header() {
        let header = ReplayHeader::new(1, "0.1.0", "battle_001", 42);
        assert_eq!(header.schema_version, 1);
        assert_eq!(header.scene_id, "battle_001");
        assert_eq!(header.initial_seed, 42);
    }

    #[test]
    fn unit_011_replay_header_add_participant() {
        let mut header = ReplayHeader::new(1, "0.1.0", "scene", 0);
        header.add_participant("unit_001");
        header.add_participant("unit_002");
        assert_eq!(header.participants.len(), 2);
    }

    #[test]
    fn unit_012_replay_header_set_total_frames() {
        let mut header = ReplayHeader::new(1, "0.1.0", "scene", 0);
        header.set_total_frames(100);
        assert_eq!(header.total_frames, 100);
    }

    #[test]
    fn unit_013_replay_error_display() {
        let err = ReplayError::VersionMismatch {
            expected: 2,
            actual: 1,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("v2"));
        assert!(msg.contains("v1"));

        let err = ReplayError::NotRecording;
        assert_eq!(format!("{}", err), "not in recording mode");
    }

    #[test]
    fn unit_014_replay_command_use_ability() {
        let cmd = ReplayCommand::UseAbility {
            caster: "unit_001".into(),
            ability_def_id: "abl_000001".into(),
            target: AbilityTarget::Single("enemy_001".into()),
        };
        assert_eq!(cmd.type_name(), "UseAbility");
    }

    #[test]
    fn unit_015_replay_command_dialogue() {
        let cmd = ReplayCommand::DialogueChoice {
            speaker: "npc_001".into(),
            choice_id: "accept_quest".into(),
        };
        assert_eq!(cmd.type_name(), "DialogueChoice");
    }

    #[test]
    fn unit_016_replay_frame_multiple_commands() {
        let mut frame = ReplayFrame::new(10, 555);
        frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });
        frame.add_command(ReplayCommand::SkipTurn { unit: "u2".into() });
        frame.add_command(ReplayCommand::SkipTurn { unit: "u3".into() });
        assert_eq!(frame.command_count(), 3);
    }
}
