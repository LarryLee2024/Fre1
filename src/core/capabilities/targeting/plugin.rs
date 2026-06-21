use bevy::prelude::*;

/// 目标选择能力插件。
///
/// 注册目标选择领域事件，供 Ability 和 UI 订阅消费。
pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
