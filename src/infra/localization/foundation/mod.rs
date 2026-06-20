//! foundation — Pure types, zero Bevy ECS dependencies.
//!
//! Contains LocaleId enum, LocError, and Pattern struct.
//! These types have no ECS Resource/Component or System dependencies.

pub(crate) mod error;
pub(crate) mod locale_id;
pub(crate) mod pattern;

pub use error::LocError;
pub use locale_id::LocaleId;
pub use pattern::Pattern;
