// UI 模块：面板、行动菜单、浮窗、视觉效果
// 架构：widgets/ 基础库 + panels/ 面板模块 + 各功能模块

mod action_menu;
mod camera;
mod combat_log_handler;
mod combat_preview;
mod combat_vfx_handler;
mod command_handler;
pub mod events;
mod focus;
mod highlight;
mod panels;
mod plugin;
pub mod settings;
pub mod theme;
mod tile_info;
pub mod vfx;
pub mod view_models;
mod widgets;

// 公共 re-exports
pub use focus::{BlocksGameInput, UiFocusState};
pub use plugin::UiPlugin;
pub use settings::GameSettings;
pub use theme::UiTheme;
