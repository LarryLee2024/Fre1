//! 游戏状态枚举定义 — GameState / OverlayState / TransitionRequest
//!
//! 定义在 shared/ 层使所有纵向层（shared/core/infra）均可引用，
//! 避免 Core → App 的层依赖违规。
//!
//! 详见 ADR-050 §1: 两层状态架构。

use bevy::prelude::*;

/// 顶层游戏状态 — 驱动全局游戏流程。
///
/// ```text
/// MainMenu → PartySetup → TacticalMap ⇄ Combat → Result
///                         ↓                     ↓
///                      CampRest              GameOver
/// ```
///
/// 切换 GameState 意味着进入一个不同的"世界模式"：
/// ECS 系统集变化、UI 集合变化、输入规则变化、摄像机规则变化。
/// 上一个场景的实体在 OnExit 时通过 `cleanup_scene` 卸载。
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    /// 主菜单/标题画面。默认启动状态。
    #[default]
    MainMenu,
    /// 队伍编成/战前准备。
    PartySetup,
    /// 战术地图（网格探索、遭遇、商店入口、对话入口）。
    TacticalMap,
    /// 战斗进行中。
    Combat,
    /// 战斗结算（胜利/失败奖励展示）。
    Result,
    /// 营地界面（短休/长休/队伍管理）。
    CampRest,
    /// 游戏结束画面。
    GameOver,
}

/// 临时覆盖层 — 叠加在当前 GameState 之上，不触发场景重建。
///
/// 与 GameState 的核心区别：覆盖层不卸载当前场景。
/// 例如 TacticalMap 中触发 Dialogue，地图保持挂载，
/// PopOverlay 后直接回到地图，无需重建。
///
/// 覆盖层不单独注册为 Bevy State，其生命周期通过
/// `StateTransitionQueue` 的 `PushOverlay` / `PopOverlay` 请求管理。
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum OverlayState {
    /// 无覆盖层。
    #[default]
    None,
    /// 对话界面（可叠加在任何 GameState 之上）。
    Dialogue,
    /// 商店界面（通常叠加在 TacticalMap 或 CampRest 之上）。
    Shop,
    /// 过场演出（可叠加在任何 GameState 之上）。
    Cutscene,
    /// 新手指引。
    Tutorial,
}

/// 状态转移请求。
///
/// 域系统通过此枚举向 `StateTransitionQueue` 提交转移请求，
/// 禁止直接调用 `NextState<GameState>`。
pub enum TransitionRequest {
    /// 切换 GameState（触发场景卸载/加载）。
    Change(GameState),
    /// 推送覆盖层（当前场景保持挂载）。
    PushOverlay(OverlayState),
    /// 弹出覆盖层（回到上一个覆盖层或无覆盖状态）。
    PopOverlay,
}
