//! Module Name: Binding — UI data binding infrastructure
//!
//! Provides the Dirty<T> change-tracking mechanism and UiBinding enum
//! that connect Projection updates (UiStore writes) to Widget refreshes.
//!
//! Dirty<T> is a Component attached to Widget entities.  Projection updates
//! mark it dirty; Widget systems call consume() to check and refresh only
//! when data has changed.
//!
//! UiBinding is a marker enum that identifies which ViewModel field a UI
//! Node is bound to.  Uses anti-Marker pattern to avoid Archetype explosion.
//!
//! See `docs/06-ui/02-design-system/focus-binding.md` §3-4

pub mod dirty_flag;
pub mod ui_binding;

pub use dirty_flag::Dirty;
pub use ui_binding::UiBinding;

#[cfg(test)]
mod tests;
