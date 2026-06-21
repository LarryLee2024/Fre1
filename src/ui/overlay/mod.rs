//! Overlay — UI 浮层系统
//!
//! Overlay 是独立于 Screen 实体的 UI 元素，拥有自己的层级和生命周期。
//! 5 层架构：Screen → Popup → Tooltip → Notification → Debug
//!
//! 详见 `docs/06-ui/03-screens/navigation-overlay.md`

pub mod damage_text;
pub mod debug;
pub mod layers;
pub mod loading;
pub mod notification;
pub mod plugin;
pub mod services;
pub mod tooltip;

pub use layers::*;
pub use notification::NotificationService;
pub use plugin::OverlayPlugin;
pub use tooltip::TooltipService;
