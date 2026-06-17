//! Pipeline Foundation — 执行管线基础类型与值对象

pub(crate) mod types;
pub(crate) mod values;

pub use types::{
    ExecutionLogEntry, FailureStrategy, PipelineContext, PipelineError, PipelineStage,
    PipelineStep, StepResult,
};
pub use values::{PipelineDefinition, PipelineState};
