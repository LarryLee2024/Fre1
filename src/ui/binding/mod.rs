//! Module Name: Binding — UI data binding infrastructure
//!
//! Provides the Dirty<T> change-tracking mechanism that connects Projection
//! updates (UiStore writes) to Widget refreshes (consumption reads).
//!
//! Dirty<T> is a Component attached to Widget entities.  Projection updates
//! mark it dirty; Widget systems call consume() to check and refresh only
//! when data has changed.
//!
//! See `docs/06-ui/02-design-system/focus-binding.md` §3

pub mod dirty_flag;

pub use dirty_flag::Dirty;
