//! Pipeline Hook — 执行生命周期回调
//!
//! 用于录制（ReplayRecorder）、调试日志（DebugLogger）、性能统计等场景。
//! Hook 是观察者，不是守门人 —— 禁止 Hook 阻断管线执行。
//!
//! 详见 ADR-044 §4

use crate::core::capabilities::runtime::pipeline::foundation::{PipelineContext, StepResult};
use crate::core::capabilities::runtime::pipeline::hooks::PipelineHook;

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
        tracing::trace!(target: "pipeline",
            hook = %self.name,
            stage = %stage,
            step = %step,
            result = ?result,
            "管线步骤已执行"
        );
    }
}
