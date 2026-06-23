//! 场景管理模块 — GameState 驱动的游戏流程编排
//!
//! 详见 ADR-050。

pub mod battle_end;
pub mod components;
pub mod game_over;
pub mod game_setup;
pub mod open_menu;
pub mod party_setup;
pub mod plugin;
pub mod queue;
pub mod register;
pub mod result;
pub mod state;
pub mod test_battle;

#[cfg(test)]
mod tests;

pub use components::SceneRoot;
pub use plugin::ScenePlugin;
pub use queue::StateTransitionQueue;
pub use state::{GameState, OverlayState, TransitionRequest};
