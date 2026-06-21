//! UiCommand — UI 层到 Domain 层的命令
//!
//! 所有用户操作必须通过 UiCommand 进入 Domain 层。
//! UiCommand 通过 `into_game_command()` 转换为 GameCommand，
//! 然后进入 ADR-043 定义的 CommandQueue。
//!
//! 转换器是 UI 层与 Command 层的唯一桥梁 (APP-VAL-03)。

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::ui::navigation::ScreenType;

/// UI 层命令，封装用户操作意图。
///
/// UiCommand 必须通过 `into_game_command()` 显式转换为 GameCommand
/// 后才能进入 CommandQueue。UI 内部导航命令（OpenScreen/CloseScreen）
/// 和当前无法映射的命令返回 None。
///
/// # 验证规则 (APP-VAL-02)
/// UiCommand 是纯数据枚举，匹配分支中不包含业务执行逻辑。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiCommand {
    // ── 战斗 ──
    /// 施放技能
    CastSkill {
        /// 技能定义 ID
        skill_def_id: String,
        /// 目标标识
        target_id: String,
        /// 施法者标识
        caster_id: String,
    },
    /// 选择目标（CharacterId）
    SelectTarget(u32),
    /// 结束当前回合
    EndTurn,
    /// 移动到网格位置
    MoveToPosition {
        /// 单位标识
        unit_id: String,
        /// 网格 X 坐标
        x: i32,
        /// 网格 Y 坐标
        y: i32,
    },

    // ── 背包 ──
    /// 使用物品
    UseItem {
        /// 物品实例 ID
        item_instance_id: String,
        /// 使用者标识
        user_id: String,
        /// 目标标识（可选）
        target_id: Option<String>,
    },
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

    // ── 任务 ──
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

    // ── 经济 ──
    /// 购买物品
    BuyItem {
        /// 物品定义 ID
        item_def_id: String,
        /// 数量
        quantity: u32,
        /// 商店标识
        shop_id: String,
    },
    /// 出售物品
    SellItem {
        /// 物品定义 ID
        item_def_id: String,
        /// 数量
        quantity: u32,
        /// 商店标识
        shop_id: String,
    },

    // ── 存档 ──
    /// 保存游戏（SaveSlot）
    SaveGame(u32),
    /// 加载游戏（SaveSlot）
    LoadGame(u32),

    // ── 系统 ──
    /// 切换暂停
    TogglePause,
    /// 打开页面
    OpenScreen(ScreenType),
    /// 关闭页面
    CloseScreen,
    /// 新游戏
    NewGame,
}

impl UiCommand {
    /// 将 UiCommand 转换为 GameCommand 以便 Domain 层执行。
    ///
    /// 返回 `None` 的场景：
    /// - UI 内部导航命令（OpenScreen、CloseScreen）
    /// - 需要调用方上下文信息才能填充的命令（如 SelectTarget）
    ///
    /// 调用方负责处理 None 情况（通常是直接执行而不经过 CommandQueue）。
    pub fn into_game_command(&self) -> Option<GameCommand> {
        match self {
            UiCommand::CastSkill {
                skill_def_id,
                target_id,
                caster_id,
            } => Some(GameCommand::CastSpell {
                caster_id: caster_id.clone(),
                spell_def_id: skill_def_id.clone(),
                target_id: target_id.clone(),
            }),
            UiCommand::SelectTarget(_) => None,
            UiCommand::EndTurn => Some(GameCommand::EndTurn {
                // 调用方应在入队前填充 unit_id
                unit_id: String::new(),
            }),
            UiCommand::MoveToPosition { unit_id, x, y } => Some(GameCommand::MoveUnit {
                unit_id: unit_id.clone(),
                path: vec![format!("{},{}", x, y)],
            }),
            UiCommand::UseItem {
                item_instance_id,
                user_id,
                target_id,
            } => Some(GameCommand::UseItem {
                user_id: user_id.clone(),
                item_instance_id: item_instance_id.clone(),
                target_id: target_id.clone(),
            }),
            UiCommand::EquipItem {
                unit_id,
                item_instance_id,
                slot_index,
            } => Some(GameCommand::EquipItem {
                unit_id: unit_id.clone(),
                item_instance_id: item_instance_id.clone(),
                slot_index: *slot_index,
            }),
            UiCommand::DropItem {
                unit_id,
                item_instance_id,
                quantity,
            } => Some(GameCommand::DropItem {
                unit_id: unit_id.clone(),
                item_instance_id: item_instance_id.clone(),
                quantity: *quantity,
            }),
            UiCommand::AcceptQuest {
                unit_id,
                quest_def_id,
            } => Some(GameCommand::AcceptQuest {
                unit_id: unit_id.clone(),
                quest_def_id: quest_def_id.clone(),
            }),
            UiCommand::AbandonQuest {
                unit_id,
                quest_def_id,
            } => Some(GameCommand::AbandonQuest {
                unit_id: unit_id.clone(),
                quest_def_id: quest_def_id.clone(),
            }),
            UiCommand::BuyItem {
                item_def_id,
                quantity,
                shop_id,
            } => Some(GameCommand::BuyItem {
                // buyer_id 由 Bridge 在入队前从上下文填充
                buyer_id: String::new(),
                item_def_id: item_def_id.clone(),
                quantity: *quantity,
                shop_id: shop_id.clone(),
            }),
            UiCommand::SellItem {
                item_def_id,
                quantity,
                shop_id,
            } => Some(GameCommand::SellItem {
                // seller_id 由 Bridge 在入队前从上下文填充
                seller_id: String::new(),
                item_def_id: item_def_id.clone(),
                quantity: *quantity,
                shop_id: shop_id.clone(),
            }),
            UiCommand::SaveGame(_) => Some(GameCommand::SaveGame),
            UiCommand::LoadGame(_) => Some(GameCommand::LoadGame),
            UiCommand::TogglePause => Some(GameCommand::OpenMenu),
            UiCommand::NewGame => Some(GameCommand::NewGame),
            // UI 内部导航命令或无需映射的命令
            UiCommand::OpenScreen(_) | UiCommand::CloseScreen => None,
        }
    }
}
