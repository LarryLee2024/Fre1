//! ui — 本地化文本的表现层。
//!
//! 包含 LocalizedText 组件和渲染系统。

pub(crate) mod components;
pub(crate) mod render;

pub use components::LocalizedText;
pub use render::render_localized_text;
