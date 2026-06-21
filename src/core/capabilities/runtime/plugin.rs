use bevy::prelude::*;

use super::pipeline::registry::PipelineRegistry;

/// Runtime 能力插件。
///
/// 管理执行管线、调度器、命令队列、注册中心和回放系统。
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        // 注册命令处理管线（PreUpdate 中 drain CommandQueue 并分派）
        app.add_plugins(super::command::plugin::CommandPlugin);

        // 初始化 PipelineRegistry（CombatPlugin 等依赖此资源）
        app.init_resource::<PipelineRegistry>();
    }
}
