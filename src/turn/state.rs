// 回合状态定义：游戏主状态、回合阶段、系统集合

use bevy::prelude::*;

/// 游戏主状态
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

/// 回合阶段（SubState，仅在 InGame 时激活）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum TurnPhase {
    #[default]
    /// 选择单位
    SelectUnit,
    /// 移动阶段
    MoveUnit,
    /// 行动菜单（右键弹出）
    ActionMenu,
    /// 选择攻击目标
    SelectTarget,
    /// 执行攻击
    ExecuteAction,
    /// 待机
    WaitAction,
    /// 回合结束
    TurnEnd,
}

/// 跨插件系统集合：显式控制 OnEnter(InGame) 生成顺序
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Camera,
    Map,
    Unit,
    Ui,
}
