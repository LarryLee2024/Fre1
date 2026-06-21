use bevy::prelude::*;

/// Runtime 能力插件。
///
/// 管理执行管线、调度器、命令队列、注册中心和回放系统。
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        // 注册命令处理管线（PreUpdate 中 drain CommandQueue 并分派）
        app.add_plugins(super::command::plugin::CommandPlugin);

        // ── 后续 Phase 注册 ──
        // PipelineState Resource + Observer 待实现
        // Scheduler Resource + 帧推进 System 待实现
    }
}
