//! Module Name: ScreenStack — Navigation stack for full-screen views
//!
//! Implements a LIFO stack of ScreenType values that tracks the navigation
//! history. Guards against duplicate top-of-stack pushes and empty-stack pops.
//! Designed to be registered as a Bevy Resource.

use bevy::prelude::*;

use super::screen_type::ScreenType;

/// LIFO navigation stack of screen types.
///
/// Stored as a Bevy Resource for ECS access. Only screen type identities
/// are tracked here; screen entities and their lifecycle are managed by
/// UiScreenState and screen-specific systems.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ScreenStack {
    /// Stack of screen types, bottom (earliest) to top (current).
    stack: Vec<ScreenType>,
}

impl ScreenStack {
    /// Creates an empty navigation stack.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes a screen onto the stack.
    ///
    /// If `screen` is already at the top of the stack, this is a no-op
    /// to prevent redundant pushes.
    pub fn push(&mut self, screen: ScreenType) {
        if self.peek() == Some(&screen) {
            return;
        }
        self.stack.push(screen);
    }

    /// Pops the top screen from the stack and returns it.
    ///
    /// Returns `None` if the stack has only one element remaining (the root
    /// screen is preserved). Returns `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<ScreenType> {
        if self.stack.len() <= 1 {
            return None;
        }
        self.stack.pop()
    }

    /// Replaces the top screen with a new one.
    ///
    /// Returns the previous top screen, or `None` if the stack was empty.
    /// If the stack is empty, this degrades to a push.
    pub fn replace(&mut self, screen: ScreenType) -> Option<ScreenType> {
        let old = self.stack.pop();
        self.stack.push(screen);
        old
    }

    /// Returns a reference to the top screen, or `None` if empty.
    pub fn peek(&self) -> Option<&ScreenType> {
        self.stack.last()
    }

    /// Returns `true` if the stack contains no screens.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the number of screens in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if the stack contains the given screen type.
    pub fn contains(&self, screen: ScreenType) -> bool {
        self.stack.contains(&screen)
    }

    /// Removes all screens from the stack.
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Returns an iterator over the screens, bottom to top.
    pub fn iter(&self) -> impl Iterator<Item = &ScreenType> {
        self.stack.iter()
    }
}

impl Default for ScreenStack {
    fn default() -> Self {
        Self::new()
    }
}
