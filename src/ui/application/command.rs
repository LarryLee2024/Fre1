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
    /// 施放技能（SkillId, target CharacterId）
    CastSkill(u32, u32),
    /// 选择目标（CharacterId）
    SelectTarget(u32),
    /// 结束当前回合
    EndTurn,
    /// 移动到网格位置（x, y）
    MoveToPosition(i32, i32),

    // ── 背包 ──
    /// 使用物品（ItemId）
    UseItem(u32),
    /// 装备物品（ItemId, slot index）
    EquipItem(u32, u32),
    /// 丢弃物品（ItemId）
    DropItem(u32),

    // ── 任务 ──
    /// 接受任务（QuestId）
    AcceptQuest(u32),
    /// 放弃任务（QuestId）
    AbandonQuest(u32),

    // ── 经济 ──
    /// 购买物品（ItemId, quantity）
    BuyItem(u32, u32),
    /// 出售物品（ItemId, quantity）
    SellItem(u32, u32),

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
    /// - 需要调用方上下文信息才能填充的命令（如 CastSkill 缺少 caster_id）
    /// - 当前 GameCommand 尚不支持的领域操作
    ///
    /// 调用方负责处理 None 情况（通常是直接执行而不经过 CommandQueue）。
    pub fn into_game_command(&self) -> Option<GameCommand> {
        match self {
            UiCommand::EndTurn => Some(GameCommand::EndTurn {
                // 调用方应在入队前填充 unit_id
                unit_id: String::new(),
            }),
            UiCommand::SaveGame(_) => Some(GameCommand::SaveGame),
            UiCommand::LoadGame(_) => Some(GameCommand::LoadGame),
            // 以下命令当前无法映射到 GameCommand：
            // CastSkill     — 需 caster_id (将在集成时从上下文获取)
            // SelectTarget  — 无对应 GameCommand
            // MoveToPosition — 无对应 GameCommand
            // UseItem       — 需 user_id/item_instance_id
            // EquipItem     — 无对应 GameCommand
            // DropItem      — 无对应 GameCommand
            // AcceptQuest   — 无对应 GameCommand
            // AbandonQuest  — 无对应 GameCommand
            // BuyItem       — 无对应 GameCommand
            // SellItem      — 无对应 GameCommand
            // TogglePause   — 无对应 GameCommand
            // OpenScreen    — UI 内部导航
            // CloseScreen   — UI 内部导航
            // NewGame       — 无对应 GameCommand
            _ => None,
        }
    }
}
