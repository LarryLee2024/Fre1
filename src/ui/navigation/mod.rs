//! 屏幕栈和 UI 导航状态
//!
//! 提供 ScreenStack（LIFO 导航历史）、ScreenType（屏幕标识符）、
//! UiScreenState（当前屏幕生命周期跟踪）和 ScreenLifecycle（生命周期阶段枚举）。
//!
//! 这些类型构成应用层中所有界面 UI 导航的基础。

mod screen_state;
mod screen_type;

pub mod screen_stack;

pub use screen_stack::ScreenStack;
pub use screen_state::{ScreenLifecycle, UiScreenState};
pub use screen_type::ScreenType;

#[cfg(test)]
mod tests;
