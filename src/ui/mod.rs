// UI 模块：面板、行动菜单、浮窗、视觉效果
// 架构：widgets/ 基础库 + panels/ 面板模块 + 各功能模块

mod action_menu;
mod camera;
mod combat_preview;
mod command_handler;
pub mod events;
mod highlight;
mod panels;
mod plugin;
pub mod theme;
mod tile_info;
pub mod vfx;
pub mod view_models;
mod widgets;

// 公共 re-exports
pub use plugin::UiPlugin;
pub use theme::UiTheme;
