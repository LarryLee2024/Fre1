//! Replay 基础类型与枚举
//!
//! 定义回放帧、录制命令、RNG 流以及领域错误。
//!
//! 详见 docs/04-data/infrastructure/replay_schema.md

use bevy::prelude::Reflect;

/// 回放帧——单帧的命令集合 + 种子信息。
#[derive(Debug, Clone, PartialEq, Reflect)]
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
#[derive(Debug, Clone, PartialEq, Reflect)]
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
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum AbilityTarget {
    /// 单体目标
    Single(String),
    /// 区域目标（位置坐标）
    Area(String),
    /// 无目标（如自我施法）
    None,
}

/// 回放头信息——回放日志元数据。
#[derive(Debug, Clone, PartialEq, Reflect)]
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
