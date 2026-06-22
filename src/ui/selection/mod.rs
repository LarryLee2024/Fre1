//! Selection — 选择状态管理（与 picking 平级）
//!
//! 包含 SelectionState 五态状态机、PickContext Resource、
//! 以及 PickIntent → Domain Event 桥接。
//!
//! 详见 ADR-068 §Module Design。

pub mod bridge;
pub mod pick_context;
pub mod plugin;
pub mod state;

pub use bridge::{SelectionCleared, TileClicked, UnitClicked};
pub use pick_context::PickContext;
pub use plugin::SelectionPlugin;
pub use state::{Selection, SelectionState};
