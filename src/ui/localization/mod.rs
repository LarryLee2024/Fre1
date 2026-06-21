//! UI Localization — 对接 infra 本地化系统
//!
//! 提供便捷工厂函数，将 UI 文本连接到 infra/localization 的
//! LocalizedText 组件和渲染管线。
//!
//! 使用方式：
//! ```ignore
//! use crate::infra::localization::generated::loc;
//! spawn_localized_text(&mut commands, &asset_server, &theme,
//!     loc::ui::BATTLE_END_TURN, TextVariant::Body);
//! ```
//!
//! 详见 `docs/06-ui/02-design-system/theme-localization.md` §4

pub mod text_keys;
