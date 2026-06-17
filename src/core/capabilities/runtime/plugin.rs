use bevy::prelude::*;

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
