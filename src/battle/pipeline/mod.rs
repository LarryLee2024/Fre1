// Effect Pipeline：生成→修饰→执行 三步管道

mod generate;
mod modify;
mod execute;
mod intent;

pub use intent::{CombatIntent, PrevPosition};
pub use execute::{
    execute_effects_inline, apply_damage_effect, apply_heal_effect, apply_buff_effect,
    apply_cleanse_effect,
};

use bevy::prelude::*;

/// 战斗事件插件：注册 Effect Pipeline 系统
pub struct CombatEventPlugin;

impl Plugin for CombatEventPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::TurnPhase;
        app.init_resource::<CombatIntent>()
            .init_resource::<PrevPosition>()
            .add_systems(
                OnEnter(TurnPhase::ExecuteAction),
                (
                    generate::generate_combat_effects,
                    modify::modify_effects,
                    execute::execute_effects,
                    intent::execute_action_on_enter,
                )
                    .chain(),
            )
            .add_systems(OnEnter(TurnPhase::WaitAction), intent::wait_action_on_enter);
    }
}
