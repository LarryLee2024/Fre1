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

    // ── Commerce ─────────────────────────────────
    /// 购买物品
    BuyItem {
        /// 购买者标识
        buyer_id: String,
        /// 物品定义 ID
        item_def_id: String,
        /// 数量
        quantity: u32,
        /// 商店标识
        shop_id: String,
    },
    /// 出售物品
    SellItem {
        /// 出售者标识
        seller_id: String,
        /// 物品定义 ID
        item_def_id: String,
        /// 数量
        quantity: u32,
        /// 商店标识
        shop_id: String,
    },

    // ── Inventory ────────────────────────────────
    /// 装备物品
    EquipItem {
        /// 单位标识
        unit_id: String,
        /// 物品实例 ID
        item_instance_id: String,
        /// 槽位索引
        slot_index: u32,
    },
    /// 丢弃物品
    DropItem {
        /// 单位标识
        unit_id: String,
        /// 物品实例 ID
        item_instance_id: String,
        /// 数量
        quantity: u32,
    },

    // ── Quest ─────────────────────────────────────
    /// 接受任务
    AcceptQuest {
        /// 单位标识
        unit_id: String,
        /// 任务定义 ID
        quest_def_id: String,
    },
    /// 放弃任务
    AbandonQuest {
        /// 单位标识
        unit_id: String,
        /// 任务定义 ID
        quest_def_id: String,
    },

    // ── System ────────────────────────────────────
    /// 新游戏
    NewGame,
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
            Self::BuyItem { .. } => "BuyItem",
            Self::SellItem { .. } => "SellItem",
            Self::EquipItem { .. } => "EquipItem",
            Self::DropItem { .. } => "DropItem",
            Self::AcceptQuest { .. } => "AcceptQuest",
            Self::AbandonQuest { .. } => "AbandonQuest",
            Self::NewGame => "NewGame",
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
