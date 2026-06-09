// UI 模块：HUD、行动菜单、地块信息、视觉效果

mod action_menu;
mod combat_preview;
mod command_handler;
pub mod events;
mod hud;
mod plugin;
pub mod theme;
mod tile_info;
pub mod vfx;
pub mod view_models;

// 公共 re-exports
pub use plugin::UiPlugin;
pub use theme::UiTheme;
