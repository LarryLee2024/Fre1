//! EntityCommandsExt -- Extension trait for [`EntityCommands`].
//!
//! Provides domain-specific method sugar on top of Bevy's `EntityCommands`,
//! internally delegating to integration-layer Facade functions.
//!
//! # Stub Status
//!
//! This is an initial stub implementation. Methods will be wired to actual
//! integration facade functions as Phase C2 progresses.

use bevy::prelude::info;
use bevy::prelude::EntityCommands;

/// Extension trait for [`EntityCommands`] providing domain-specific operations.
///
/// All methods internally delegate to integration-layer Facade functions,
/// never directly manipulate Capabilities internals.
///
/// # Usage
///
/// ```ignore
/// use crate::core::domains::combat::integration::ext::EntityCommandsExt;
///
/// fn my_system(mut commands: Commands) {
///     let mut entity = commands.spawn_empty();
///     entity.add_buff(spec_id);
///     entity.heal(50);
/// }
/// ```
pub trait EntityCommandsExt {
    /// Add a buff (active effect) to this entity.
    fn add_buff(&mut self, buff_id: &str);

    /// Heal this entity for the given amount.
    fn heal(&mut self, amount: u32);

    /// Kill this entity (mark as dead).
    fn kill(&mut self);
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn add_buff(&mut self, buff_id: &str) {
        // Stub: logs the add_buff action.
        // Will be wired to EffectFacade in future Phase C2 work.
        info!("Adding buff {} (stub)", buff_id);
    }

    fn heal(&mut self, amount: u32) {
        // Stub: logs the heal action.
        // Will be wired to EffectFacade or ExecutionFacade in future Phase C2 work.
        info!("Healing entity for {} (stub)", amount);
    }

    fn kill(&mut self) {
        // Stub: logs the kill action.
        // Will wire to a Dead tag component or combat death pipeline in future.
        info!("Killing entity (stub)");
    }
}
