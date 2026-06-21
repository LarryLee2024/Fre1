//! Module Name: Focus — 焦点导航系统
//!
//! 提供键盘/手柄导航支持，包括 Focusable（可聚焦标记）、
//! FocusGroup（焦点组管理）、FocusManager（全局焦点状态）、
//! TabIndex（组内排序）和焦点视觉效果。
//!
//! 使用我们自己的 TabIndex 组件进行方向键导航排序，
//! 与 Bevy 内置 Tab/Shift+Tab 导航共存。
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §2

pub mod components;
pub mod manager;
pub mod navigation;
pub mod plugin;

pub use components::*;
pub use manager::*;
pub use navigation::*;
pub use plugin::FocusPlugin;

#[cfg(test)]
mod tests;
