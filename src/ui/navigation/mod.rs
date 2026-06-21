//! Module Name: Navigation — Screen stack and UI navigation state
//!
//! Provides ScreenStack (LIFO navigation history), ScreenType (screen
//! identifiers), UiScreenState (current screen lifecycle tracking), and
//! ScreenLifecycle (lifecycle phase enum).
//!
//! These types form the foundation for UI navigation across all screens
//! in the application layer.

mod screen_state;
mod screen_type;

pub mod screen_stack;

pub use screen_state::{ScreenLifecycle, UiScreenState};
pub use screen_type::ScreenType;
pub use screen_stack::ScreenStack;
