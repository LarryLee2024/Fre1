//! Module Name: UI — 表现层 (L3)
//!
//! 最高层，依赖 Core（L1）和 Infra（L2），但不被任何下层依赖。
//! 职责：将领域状态投影为 UI，管理 Screen/Widget/Overlay。
//!
//! 详见 `docs/06-ui/` 架构文档

pub mod plugin;
pub mod theme;
pub mod widgets;

pub use plugin::UiPlugin;
pub use theme::{Theme, ThemePlugin, UiColors, UiSpacing, UiTypography};
