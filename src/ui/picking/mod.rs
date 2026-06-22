//! Picking — Presentation Layer 输入适配子层
//!
//! 只负责命中检测后端 + PickIntent 生产。
//! 不包含任何业务逻辑或 Selection 状态管理。
//!
//! 详见 ADR-068 §Module Design。

pub mod backend;
pub mod intent;
pub mod pick_target;
pub mod plugin;

pub use pick_target::{InteractionPhase, PickIntent, PickTarget};
pub use plugin::PickingUiPlugin;
