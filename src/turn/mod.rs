// 回合管理模块：状态机、敏捷驱动行动队列、SystemSet 编排

mod order;
mod state;

pub use order::*;
pub use state::*;

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
            .add_message::<TurnStarted>()
            .add_message::<TurnEnded>()
            .add_message::<ForceEndTurn>()
            .configure_sets(
                OnEnter(AppState::InGame),
                (GameSet::Camera, GameSet::Map, GameSet::Unit, GameSet::Ui).chain(),
            )
            .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter)
            .add_systems(
                OnEnter(AppState::InGame),
                init_turn_order.after(GameSet::Unit),
            );
    }
}
