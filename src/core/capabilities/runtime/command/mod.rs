//! command — 业务命令层
//!
//! C3 Runtime 的子模块：定义统一的业务命令枚举（GameCommand）、
//! 命令队列（CommandQueue）、命令验证和分发逻辑。
//!
//! 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

pub mod events;
pub mod foundation;
pub mod mechanism;

#[cfg(test)]
mod tests;
