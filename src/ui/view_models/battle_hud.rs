//! BattleHudVm — 战斗 HUD 视图模型
//!
//! Widget 的唯一数据源。通过 UiStore 注入 Widget 系统。
//! Projection 纯函数将 Domain Event 投影为此 ViewModel。
//!
//! 所有字段为简单类型 —— 不包含任何 Domain 类型引用。
//! 文本字段使用 &'static str 作为 UiTextKey。
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3
//!
//! BattleHudData — 临时过渡 Resource
//! 在 Projection 系统完全就绪前，作为 UI 层的数据源桥接。
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §5

use bevy::prelude::*;

/// 目标选择模式
///
/// 定义玩家当前处于何种目标选择状态。
/// None 为非选择模式，Attack 为攻击目标选择模式。
#[derive(Clone, Copy, PartialEq, Reflect, Default)]
pub enum TargetingMode {
    /// 非目标选择模式
    #[default]
    None,
    /// 攻击目标选择模式
    Attack,
}

/// 战斗 HUD 视图模型
#[derive(Clone, Reflect, Default)]
pub struct BattleHudVm {
    /// 当前 HP
    pub hp: f32,
    /// 最大 HP
    pub max_hp: f32,
    /// 当前 MP
    pub mp: f32,
    /// 最大 MP
    pub max_mp: f32,
    /// 当前 AP
    pub ap: f32,
    /// 最大 AP
    pub max_ap: f32,
    /// 当前回合数
    pub turn_number: u32,
    /// 阶段描述（本地化 Key）
    pub phase_key: &'static str,
    /// 当前行动单位的 Entity 位表示 (Entity::to_bits)
    /// 0 表示无当前单位
    pub current_unit_id: u64,
    /// 是否为玩家控制回合
    pub is_player_controlled: bool,
    /// 是否处于战斗阶段 (BattlePhase::Battle)
    /// 用于控制战斗区域可见性
    pub is_in_battle: bool,
    /// 战斗是否已结束（胜负判定完成）
    /// 为 true 时，仅 Z2 显示结果文本，其余区域隐藏
    pub is_game_over: bool,
    /// 战斗结果文本（本地化 Key）
    /// 仅在 is_game_over 为 true 时有效
    pub result_key: &'static str,
    /// 技能面板是否打开
    ///
    /// 控制 Z7 内 SkillPanel 的可见性。当选中一个单位时自动为 true，
    /// 清除选择时自动为 false。未来由 ActionMenu.Skill 按钮精确控制。
    pub skill_panel_open: bool,
    /// 当前目标选择模式
    ///
    /// None 为非选择模式，Attack 为攻击目标选择模式。
    /// 由 ActionMenu Attack 按钮控制进入，确认或取消时退出。
    pub targeting_mode: TargetingMode,
}

/// 战斗 HUD 临时数据源
///
/// 在 Projection 系统完全就绪前，作为 UI 层的数据源桥接。
/// spawn_battle_screen 从此 Resource 读取数据传入 Widget 工厂，
/// 避免硬编码。
///
/// # 架构
/// 短期过渡方案。当完整的 UiBinding + Projection 管线激活后，
/// 此 Resource 将由 Projection 驱动更新或直接废弃。
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct BattleHudData {
    /// 角色名称
    pub character_name: String,
    /// 等级
    pub level: u32,
    /// 当前 HP
    pub hp_current: f32,
    /// 最大 HP
    pub hp_max: f32,
    /// 当前 MP
    pub mp_current: f32,
    /// 最大 MP
    pub mp_max: f32,
}
