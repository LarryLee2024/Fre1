//! Pipeline Foundation — 执行管线基础类型与值对象

pub mod types;
pub mod values;

pub use types::{
    ExecutionLogEntry, FailureStrategy, PipelineContext, PipelineError, PipelineStage,
    PipelineStep, StepResult,
};
pub use values::{PipelineDefinition, PipelineState};
