//! save — 基础设施存档层
//!
//! 提供存档系统的 ECS 基础设施：
//! - Resource: SaveManager, AutoSaveConfig, EntityRemapper
//! - Events: SaveRequest, LoadRequest, SaveCompleted, LoadCompleted
//! - Observer: on_save_request, on_load_request
//!
//! 当前为最小实现（桥接层骨架），后续迭代接入 Per-Feature 序列化。
//!
//! 详见 ADR-042

mod plugin;
pub(crate) mod resources;
pub(crate) mod systems;

mod events;

#[cfg(test)]
#[cfg(test)]
mod tests;

pub use events::{
    LoadCompleted, LoadRequest, SaveCompleted, SaveError, SaveOperation, SaveRequest,
};
pub use plugin::*;
pub use resources::{
    AutoSaveConfig, EntityRemapper, PersistentEntityId, SaveManager, SaveMetadata,
};
