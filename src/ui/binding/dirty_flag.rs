//! Module Name: Dirty<T> — Dirty flag mechanism for ViewModel change tracking
//!
//! Dirty<T> wraps a ViewModel value and tracks whether it has been modified.
//! Widgets check the dirty flag each frame and only refresh when it is set,
//! avoiding per-frame full traversal of all widgets.
//!
//! Usage:
//!   - Projection updates ViewModel via get_mut() (auto-marks dirty)
//!   - Widget system calls consume() to check and clear the flag
//!   - consume() returns true only once per dirty transition
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §5

use bevy::prelude::*;

/// Dirty flag wrapper for ViewModel change tracking.
///
/// Widgets detect the dirty flag each frame and refresh only when it is true.
/// consume() clears the flag automatically, preventing duplicate consumption
/// within the same frame.
///
/// # Contract
/// - `get_mut()` auto-marks dirty -- no separate `mark_dirty()` call needed
///   when writing through `get_mut()`
/// - Widgets **must** call `consume()` and only refresh when it returns true
/// - `mark_dirty()` is for cases where the inner value is replaced entirely
///   (e.g., UiStore field swap on screen restore)
#[derive(Component, Debug, Clone, Reflect)]
pub struct Dirty<T: Reflect + Default + Clone + 'static> {
    pub inner: T,
    is_dirty: bool,
}

impl<T: Reflect + Default + Clone + 'static> Dirty<T> {
    /// Creates a new Dirty<T> with the inner value and dirty flag set to true.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            is_dirty: true,
        }
    }

    /// Explicitly marks the value as dirty.
    ///
    /// Called by Projection when replacing UiStore fields entirely.
    /// When mutating fields through `get_mut()`, dirty is set automatically.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Consumes the dirty flag.
    ///
    /// Returns true if the value was dirty (caller should refresh), then
    /// clears the flag.  Returns false if already consumed this frame.
    /// Each dirty transition triggers at most one refresh.
    pub fn consume(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            return true;
        }
        false
    }

    /// Returns an immutable reference to the inner value.
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner value, automatically marking
    /// it dirty.
    pub fn get_mut(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.inner
    }
}

impl<T: Reflect + Default + Clone + 'static> Default for Dirty<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
