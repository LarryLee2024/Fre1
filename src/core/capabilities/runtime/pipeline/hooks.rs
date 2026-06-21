//! Pipeline Hook — 执行生命周期回调
//!
//! 用于录制（ReplayRecorder）、调试日志（DebugLogger）、性能统计等场景。
//! Hook 是观察者，不是守门人 —— 禁止 Hook 阻断管线执行。
//!
//! 详见 ADR-044 §4

use super::foundation::{PipelineContext, StepResult};

/// Sealed trait — 防止外部实现破坏 PipelineHook 的不变量。
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Pipeline Hook — 前置/后置回调
///
/// 所有方法都有默认空实现，实现者只需覆盖关心的方法。
///
/// 🟥 禁止 Hook 修改 PipelineContext 中的业务数据。
/// 🟥 禁止 Hook 返回 Result 阻断执行。
pub trait PipelineHook: sealed::Sealed + Send + Sync {
    /// Hook 名称（用于日志和调试）
    fn name(&self) -> &str;

    /// 在 Stage 执行前调用
    fn on_stage_start(&self, _stage: &str, _context: &PipelineContext) {}

    /// 在 Stage 执行后调用
    fn on_stage_end(&self, _stage: &str, _context: &PipelineContext, _result: &StepResult) {}

    /// 在 Step 执行前调用
    fn on_step_start(&self, _stage: &str, _step: &str, _context: &PipelineContext) {}

    /// 在 Step 执行后调用
    fn on_step_end(
        &self,
        _stage: &str,
        _step: &str,
        _context: &PipelineContext,
        _result: &StepResult,
    ) {
    }
}
