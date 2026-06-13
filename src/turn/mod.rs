/// 回合管理模块：状态机、行动队列、胜负检查、SystemSet 编排

/// TurnOrder 行动队列（敏捷驱动排序）
mod order;
/// AppState, TurnPhase, GameOverState 状态定义
mod state;
/// 胜负条件检查系统（数据驱动，读取 LevelConfig.VictoryConditionDef）
mod victory_check;

pub use order::*;
pub use state::*;
// LevelCompleted 供 battle/record、ui 等外部模块消费
#[allow(unused_imports)]
pub use victory_check::LevelCompleted;

use bevy::prelude::*;

/// 回合管理插件
pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<TurnPhase>()
            .init_resource::<TurnState>()
            .init_resource::<TurnOrder>()
            .init_resource::<AiTimer>()
            .init_resource::<NeedsResolve>()
            .init_resource::<GameOverState>()
            // 注册 Reflect 类型
            .register_type::<TurnPhase>()
            .register_type::<TurnOrder>()
            .register_type::<TurnState>()
            .register_type::<AiTimer>()
            .register_type::<NeedsResolve>()
            .register_type::<GameOverState>()
            .add_message::<TurnStarted>()
            .add_message::<TurnEnded>()
            .add_message::<ForceEndTurn>()
            .add_message::<victory_check::LevelCompleted>()
            .configure_sets(
                OnEnter(AppState::InGame),
                (GameSet::Camera, GameSet::Map, GameSet::Unit, GameSet::Ui).chain(),
            )
            // 胜负检查在回合结束处理之前运行（读取当前回合号，检查完再递增）
            .add_systems(
                OnEnter(TurnPhase::TurnEnd),
                (
                    victory_check::check_victory_conditions.before(turn_end_on_enter),
                    turn_end_on_enter,
                ),
            )
            // 兜底系统：全灭玩家即失败（绝对不变量防御性保障）
            // 仅在 Playing 状态时运行，终态后自动跳过（性能优化）
            .add_systems(
                Update,
                victory_check::check_all_dead_safety
                    .run_if(in_state(AppState::InGame))
                    .run_if(|game_over: Res<GameOverState>| *game_over == GameOverState::Playing),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                init_turn_order.after(GameSet::Unit),
            );
    }
}
