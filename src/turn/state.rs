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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Reflect, SubStates)]
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

/// 胜负状态（游戏终态标识，UI 层只读）
///
/// Playing → Victory/Defeat 为不可逆转换。
/// 由 `check_victory_conditions` 系统写入，UI 层仅读取展示。
#[derive(Resource, Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Resource)]
pub enum GameOverState {
    /// 战斗进行中
    #[default]
    Playing,
    /// 玩家胜利（终态，不可逆）
    Victory,
    /// 玩家失败（终态，不可逆）
    Defeat,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test ID: UI-GAME-001 (migrated from view_models)
    /// Title: GameOverState 默认值为 Playing
    #[test]
    fn game_over_state_default_is_playing() {
        let state = GameOverState::default();
        assert_eq!(state, GameOverState::Playing);
    }

    /// Test ID: UI-GAME-002 (migrated from view_models)
    /// Title: GameOverState 枚举值可比较
    #[test]
    fn game_over_state_variants_are_distinct() {
        let victory = GameOverState::Victory;
        let defeat = GameOverState::Defeat;
        assert_ne!(victory, defeat);
        assert_ne!(victory, GameOverState::Playing);
        assert_ne!(defeat, GameOverState::Playing);
    }
}
