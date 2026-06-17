//! input — 输入基础设施层
//!
//! Layer 1: Raw Input → InputAction（通过 InputMap 翻译）
//! Layer 2: InputAction → GameCommand（由各 Domain 的 System 完成）
//! Layer 3: GameCommand → CommandQueue（统一入口）
//!
//! 详见 docs/04-data/infrastructure/input_schema.md
//! 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

pub mod action;
mod plugin;
pub mod resources;
pub mod systems;

pub use plugin::*;

#[cfg(test)]
mod tests;
