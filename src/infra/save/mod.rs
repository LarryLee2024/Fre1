//! save — 基础设施存档层
//!
//! 提供存档系统的 ECS 基础设施：
//! - Resource: SaveManager, AutoSaveConfig, EntityRemapper
//! - Events: SaveRequest, LoadRequest, SaveCompleted, LoadCompleted
//! - Systems: save_world_system, on_load_request, process_pending_load
//!
//! 详见 ADR-042

pub(crate) mod load_system;
mod plugin;
pub(crate) mod resources;
pub(crate) mod save_data;
pub(crate) mod save_system;
pub(crate) mod systems;

mod events;

#[cfg(test)]
mod tests;

pub use events::{
    LoadCompleted, LoadRequest, SaveCompleted, SaveError, SaveOperation, SaveRequest,
};
pub use plugin::*;
pub use resources::{
    AutoSaveConfig, EntityRemapper, PersistentEntityId, SaveManager, SaveMetadata,
};
