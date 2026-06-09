// Effect Pipeline：生成→修饰→执行 三步管道

mod execute;
mod generate;
mod intent;
mod modify;

pub use intent::{CombatIntent, PrevPosition};

use bevy::prelude::*;
use super::events::on_character_died;

/// 战斗事件插件：注册 Effect Pipeline 系统 + 死亡消息响应
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
            .add_systems(OnEnter(TurnPhase::WaitAction), intent::wait_action_on_enter)
            // 死亡消息响应：每帧检测 CharacterDied 消息
            .add_systems(Update, on_character_died);
    }
}
