use bevy::prelude::*;

/// Runtime 能力插件。
///
/// 管理执行管线、调度器、命令队列、注册中心和回放系统。
/// 当前 Phase B 状态：领域层纯函数已完成，ECS 集成待后续 Phase 实现。
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, _app: &mut App) {
        // ── 当前 Phase B 状态 ──────────────────────────────────
        //
        // Runtime 子模块（pipeline/scheduler/registry/command/replay）
        // 领域层纯函数实现已完成，全部 Event 类型已定义：
        //   - PipelineStarted / PipelineStepCompleted / PipelineFailed / PipelineCompleted
        //   - CommandEvents: CommandEnqueued / CommandExecuted / CommandFailed
        //   - RegistryEvents: Registered / Resolved / RegistrationFailed
        //   - TickEvent
        //
        // ECS 集成注册时机（将在后续 Phase 实现）：
        //   1. PipelineState Resource → app.init_resource::<PipelineState>()
        //   2. app.add_observer(on_pipeline_started)    — 初始化管线执行上下文
        //   3. app.add_observer(on_pipeline_completed)   — 清理/触发回调
        //   4. Scheduler Resource + 帧推进 System
        //
        // 详见 docs/01-architecture/README.md §3.2
    }
}
