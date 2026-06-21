use bevy::prelude::*;

/// 游戏上下文能力插件。
///
/// GameplayContext 是纯数据载体，无 ECS Resource 或 System。
/// 注册领域事件供 Ability/Execution/Cue 订阅消费。
pub struct GameplayContextPlugin;

impl Plugin for GameplayContextPlugin {
    fn build(&self, _app: &mut App) {
        // GameplayContext is a pure data carrier with no ECS resources or systems.
        // Events use Bevy 0.19 observer pattern — derived with #[derive(Event)];
        // no explicit registration needed.
    }
}
