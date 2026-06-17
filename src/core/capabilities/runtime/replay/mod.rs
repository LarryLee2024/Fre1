//! replay — 回放系统
//!
//! C3 Runtime 的子模块：提供确定性回放的完整基础设施，
//! 包括回放录制器（Recorder）、播放器（Player）、
//! 确定性 RNG（DeterministicRng）和校验验证器（Validator）。
//!
//! 详见 docs/04-data/infrastructure/replay_schema.md
//! 详见 docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md

pub mod events;
pub mod foundation;
pub mod mechanism;
