use bevy::prelude::*;

/// Execution 能力插件——注册执行计算相关事件。
///
/// 当前为空实现，事件通过 commands.trigger() 触发。
pub struct ExecutionPlugin;

impl Plugin for ExecutionPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
