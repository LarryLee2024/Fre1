//! BattleHudVm — 战斗 HUD 视图模型
//!
//! Widget 的唯一数据源。通过 UiStore 注入 Widget 系统。
//! Projection 纯函数将 Domain Event 投影为此 ViewModel。
//!
//! 所有字段为简单类型 —— 不包含任何 Domain 类型引用。
//! 文本字段使用 &'static str 作为 UiTextKey。
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use bevy::prelude::*;

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
}
