// UI 模块：HUD、行动菜单、地块信息、视觉效果
// 合并了原 ui.rs、action_menu.rs、tile_info.rs、vfx.rs

mod hud;
mod action_menu;
mod tile_info;
pub mod vfx;
mod plugin;

// 公共 re-exports
pub use hud::*;
pub use action_menu::*;
pub use tile_info::*;
pub use vfx::*;
pub use plugin::UiPlugin;
