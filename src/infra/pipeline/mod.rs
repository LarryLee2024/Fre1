//! pipeline — 执行管线基础设施层
//!
//! 提供 Pipeline 的 ECS 集成层：PipelineRegistry Resource、PipelineHook 机制、
//! PipelinePlugin 接线。底层的纯逻辑（类型定义、执行引擎）位于
//! `core::capabilities::runtime::pipeline/`。
//!
//! 外部使用者通过此处访问管线能力：
//! ```rust
//! use fre::infra::pipeline::{PipelineRegistry, PipelineDefinition, execute_pipeline};
//! ```
//!
//! 详见 ADR-044
//! 详见 docs/04-data/infrastructure/pipeline_schema.md

mod plugin;
pub use plugin::*;

mod registry;
pub use registry::*;

pub mod hooks;

// ── Re-export core pipeline types for external consumption ──
pub use crate::core::capabilities::runtime::pipeline::events::{
    PipelineCompleted, PipelineFailed, PipelineStarted, PipelineStepCompleted,
};
pub use crate::core::capabilities::runtime::pipeline::foundation::{
    ExecutionLogEntry, FailureStrategy, PipelineContext, PipelineDefinition, PipelineError,
    PipelineStage, PipelineState, PipelineStep, StepResult,
};
pub use crate::core::capabilities::runtime::pipeline::mechanism::executor::{
    execute_pipeline, validate_pipeline,
};

#[cfg(test)]
mod tests;
