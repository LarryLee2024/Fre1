//! Command 基础类型与枚举
//!
//! 定义业务命令（GameCommand）、命令来源以及领域错误。
//!
//! 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

/// 命令来源——标识命令是来自玩家、AI 还是回放。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandSource {
    /// 玩家输入
    Player,
    /// AI 决策
    AI,
    /// 回放系统
    Replay,
    /// 系统内部
    System,
}

impl CommandSource {
    /// 返回来源名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Player => "Player",
            Self::AI => "AI",
            Self::Replay => "Replay",
            Self::System => "System",
        }
    }
}

/// 业务命令——所有玩家/AI/Replay 操作的统一枚举。
///
/// 执行系统不区分命令来源，统一处理此枚举。
#[derive(Debug, Clone, PartialEq)]
pub enum GameCommand {
    // ── Tactical ──────────────────────────────────
    /// 单位移动
    MoveUnit {
        /// 单位标识
        unit_id: String,
        /// 移动路径（坐标字符串数组）
        path: Vec<String>,
    },
    /// 等待/待机
    Wait {
        /// 单位标识
        unit_id: String,
    },

    // ── Combat ────────────────────────────────────
    /// 攻击
    Attack {
        /// 攻击者标识
        attacker_id: String,
        /// 目标标识
        target_id: String,
        /// 能力槽位（可选，空为普攻）
        ability_slot: Option<u32>,
    },
    /// 施放法术
    CastSpell {
        /// 施法者标识
        caster_id: String,
        /// 法术定义 ID
        spell_def_id: String,
        /// 目标标识
        target_id: String,
    },
    /// 使用物品
    UseItem {
        /// 使用者标识
        user_id: String,
        /// 物品实例 ID
        item_instance_id: String,
        /// 目标标识（可选）
        target_id: Option<String>,
    },

    // ── Turn ──────────────────────────────────────
    /// 结束当前回合
    EndTurn {
        /// 当前单位标识
        unit_id: String,
    },

    // ── Meta ──────────────────────────────────────
    /// 打开菜单
    OpenMenu,
    /// 保存游戏
    SaveGame,
    /// 加载游戏
    LoadGame,
}

impl GameCommand {
    /// 返回命令的名称标识。
    pub fn name(&self) -> &str {
        match self {
            Self::MoveUnit { .. } => "MoveUnit",
            Self::Wait { .. } => "Wait",
            Self::Attack { .. } => "Attack",
            Self::CastSpell { .. } => "CastSpell",
            Self::UseItem { .. } => "UseItem",
            Self::EndTurn { .. } => "EndTurn",
            Self::OpenMenu => "OpenMenu",
            Self::SaveGame => "SaveGame",
            Self::LoadGame => "LoadGame",
        }
    }
}

/// 录制命令——包装了 GameCommand 和来源、时间戳的录制格式。
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedCommand {
    /// 命令来源
    pub source: CommandSource,
    /// 原始命令
    pub command: GameCommand,
    /// 帧序号
    pub frame_number: u64,
}

impl RecordedCommand {
    /// 创建录制命令。
    pub fn new(source: CommandSource, command: GameCommand, frame_number: u64) -> Self {
        Self {
            source,
            command,
            frame_number,
        }
    }
}

/// Command 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    /// 命令队列已满
    QueueFull(usize),
    /// 命令无效
    InvalidCommand(String),
    /// 命令执行失败
    ExecutionFailed { command: String, reason: String },
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull(size) => write!(f, "command queue full (max {})", size),
            Self::InvalidCommand(msg) => write!(f, "invalid command: {}", msg),
            Self::ExecutionFailed { command, reason } => {
                write!(f, "execution failed for '{}': {}", command, reason)
            }
        }
    }
}

impl std::error::Error for CommandError {}

/// 命令分发结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchResult {
    /// 已分发给对应处理器
    Dispatched,
    /// 无处理器处理此命令
    Unhandled(String),
    /// 分发失败
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_command_source_name() {
        assert_eq!(CommandSource::Player.name(), "Player");
        assert_eq!(CommandSource::AI.name(), "AI");
        assert_eq!(CommandSource::Replay.name(), "Replay");
        assert_eq!(CommandSource::System.name(), "System");
    }

    #[test]
    fn unit_002_game_command_move_unit() {
        let cmd = GameCommand::MoveUnit {
            unit_id: "unit_001".into(),
            path: vec!["0,0".into(), "0,1".into()],
        };
        assert_eq!(cmd.name(), "MoveUnit");
    }

    #[test]
    fn unit_003_game_command_wait() {
        let cmd = GameCommand::Wait {
            unit_id: "unit_001".into(),
        };
        assert_eq!(cmd.name(), "Wait");
    }

    #[test]
    fn unit_004_game_command_attack() {
        let cmd = GameCommand::Attack {
            attacker_id: "unit_001".into(),
            target_id: "unit_002".into(),
            ability_slot: None,
        };
        assert_eq!(cmd.name(), "Attack");
    }

    #[test]
    fn unit_005_game_command_cast_spell() {
        let cmd = GameCommand::CastSpell {
            caster_id: "unit_001".into(),
            spell_def_id: "spl_000001".into(),
            target_id: "unit_002".into(),
        };
        assert_eq!(cmd.name(), "CastSpell");
    }

    #[test]
    fn unit_006_game_command_use_item() {
        let cmd = GameCommand::UseItem {
            user_id: "unit_001".into(),
            item_instance_id: "itm_000001".into(),
            target_id: None,
        };
        assert_eq!(cmd.name(), "UseItem");
    }

    #[test]
    fn unit_007_game_command_end_turn() {
        let cmd = GameCommand::EndTurn {
            unit_id: "unit_001".into(),
        };
        assert_eq!(cmd.name(), "EndTurn");
    }

    #[test]
    fn unit_008_game_command_meta() {
        assert_eq!(GameCommand::OpenMenu.name(), "OpenMenu");
        assert_eq!(GameCommand::SaveGame.name(), "SaveGame");
        assert_eq!(GameCommand::LoadGame.name(), "LoadGame");
    }

    #[test]
    fn unit_009_recorded_command() {
        let cmd = GameCommand::Wait {
            unit_id: "unit_001".into(),
        };
        let recorded = RecordedCommand::new(CommandSource::Player, cmd, 42);
        assert_eq!(recorded.source, CommandSource::Player);
        assert_eq!(recorded.frame_number, 42);
        assert_eq!(recorded.command.name(), "Wait");
    }

    #[test]
    fn unit_010_error_display() {
        let err = CommandError::QueueFull(128);
        let msg = format!("{}", err);
        assert!(msg.contains("128"));

        let err = CommandError::InvalidCommand("unknown action".into());
        let msg = format!("{}", err);
        assert!(msg.contains("unknown action"));
    }
}
