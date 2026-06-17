//! pipeline — 执行管线
//!
//! C3 Runtime 的子模块：定义可编排的执行管线，
//! 包含阶段（Stage）、步骤（Step）、失败策略（FailureStrategy）。
//!
//! 详见 docs/04-data/infrastructure/pipeline_schema.md

pub mod events;
pub mod foundation;
pub mod mechanism;

#[cfg(test)]
mod tests;
