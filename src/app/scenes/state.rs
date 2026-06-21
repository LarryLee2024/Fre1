//! 游戏状态枚举定义 — 重新导出自 shared/ 层
//!
//! 保持 `app::scenes::state` 的路径兼容性。
//! 类型定义在 `src/shared/game_state.rs`。
//!
//! 详见 ADR-050 §1: 两层状态架构。

pub use crate::shared::game_state::{GameState, OverlayState, TransitionRequest};
