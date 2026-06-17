//! Pipeline Hook — 执行生命周期回调
//!
//! 用于录制（ReplayRecorder）、调试日志（DebugLogger）、性能统计等场景。
//! Hook 是观察者，不是守门人 —— 禁止 Hook 阻断管线执行。
//!
//! 详见 ADR-044 §4

use crate::core::capabilities::runtime::pipeline::foundation::{PipelineContext, StepResult};

/// Pipeline Hook — 前置/后置回调
///
/// 所有方法都有默认空实现，实现者只需覆盖关心的方法。
///
/// 🟥 禁止 Hook 修改 PipelineContext 中的业务数据。
/// 🟥 禁止 Hook 返回 Result 阻断执行。
pub trait PipelineHook: Send + Sync {
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

// ─── 内置 Hook ──────────────────────────────────────────────

/// 执行日志 Hook — 将管线执行日志写入指定目标
///
/// 当前简化实现：trace! 级别日志输出。
/// 后续可扩展为写入 ExecutionLog Resource 或文件。
pub struct ExecutionLogHook {
    name: String,
}

impl ExecutionLogHook {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl PipelineHook for ExecutionLogHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_step_end(
        &self,
        stage: &str,
        step: &str,
        _context: &PipelineContext,
        result: &StepResult,
    ) {
        tracing::trace!(
            hook = %self.name,
            stage = %stage,
            step = %step,
            result = ?result,
            "pipeline step executed"
        );
    }
}
