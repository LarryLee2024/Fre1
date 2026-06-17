/// Effect Pipeline：生成→修饰→执行 三步管道

/// execute_effects 执行系统（apply_effects 子系统）
mod execute;
/// 效果生成（CombatIntent → PendingEffect）
mod generate;
/// CombatIntent, PrevPosition 定义
mod intent;
/// 修饰符管线（Modifier → Final Stat）
mod modify;
/// on_attack/on_hit/on_kill Trait 触发
mod trait_trigger;

pub use execute::execute_effects;
pub use intent::{CombatIntent, PrevPosition};
pub use trait_trigger::{trigger_on_attack_traits, trigger_on_hit_traits, trigger_on_kill_traits};

use super::events::on_character_died;
use bevy::prelude::*;

/// 战斗事件插件：注册 Effect Pipeline 系统 + 死亡消息响应
pub struct CombatEventPlugin;

impl Plugin for CombatEventPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::TurnPhase;
        app.init_resource::<CombatIntent>()
            .init_resource::<PrevPosition>()
            // 生成→修饰 使用 chain 确保顺序
            .add_systems(
                OnEnter(TurnPhase::ExecuteAction),
                (generate::generate_combat_effects, modify::modify_effects).chain(),
            )
            // 执行效果：使用 world: &mut World 参数，不能与 chain 一起使用
            // 必须在 generate + modify 之后运行，通过 .after 保证顺序
            .add_systems(
                OnEnter(TurnPhase::ExecuteAction),
                (execute::execute_effects, intent::execute_action_on_enter)
                    .chain()
                    .after(modify::modify_effects),
            )
            .add_systems(OnEnter(TurnPhase::WaitAction), intent::wait_action_on_enter)
            // 死亡消息响应：每帧检测 CharacterDied 消息
            .add_systems(Update, on_character_died);
    }
}
