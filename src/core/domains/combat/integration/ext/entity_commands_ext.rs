//! EntityCommandsExt -- Extension trait for [`EntityCommands`].
//!
//! Provides domain-specific method sugar on top of Bevy's `EntityCommands`,
//! internally delegating to integration-layer Facade functions.
//!
//! All methods return `&mut Self` to support chainable DSL usage.

use bevy::prelude::info;
use bevy::prelude::EntityCommands;

use crate::core::domains::combat::components::Dead;

/// Extension trait for [`EntityCommands`] providing domain-specific operations.
///
/// All methods internally delegate to integration-layer Facade functions,
/// never directly manipulate Capabilities internals.
///
/// Methods return `&mut Self` for chainable DSL usage.
///
/// # Usage
///
/// ```ignore
/// use crate::core::domains::combat::integration::ext::EntityCommandsExt;
///
/// fn my_system(mut commands: Commands) {
///     let mut entity = commands.spawn_empty();
///     entity
///         .add_buff("eff_000001")
///         .heal(50)
///         .kill();
/// }
/// ```
pub trait EntityCommandsExt {
    /// Add a buff (active effect) to this entity.
    ///
    /// Internally queues a buff request that will be processed by the
    /// Effect capability's integration facade.
    ///
    /// # Parameters
    ///
    /// * `buff_id` — The Def ID of the buff to apply (e.g. `"eff_000001"`).
    fn add_buff(&mut self, buff_id: &str) -> &mut Self;

    /// Heal this entity for the given amount.
    ///
    /// Internally delegates to the Execution integration facade which
    /// routes through the Effect/Modifier pipeline.
    ///
    /// # Parameters
    ///
    /// * `amount` — Raw heal amount before modifiers/mitigation.
    fn heal(&mut self, amount: u32) -> &mut Self;

    /// Kill this entity (mark as dead).
    ///
    /// Inserts the [`Dead`] marker component, which the combat pipeline
    /// uses to identify eliminated participants.
    fn kill(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_> {

    fn add_buff(&mut self, buff_id: &str) -> &mut Self {
        info!(
            "EntityCommandsExt::add_buff(buff_id={}) — queuing buff application",
            buff_id,
        );
        // TODO[Phase C2]: Wire to EffectFacade::apply_buff once the effect
        //   integration facade exposes a command-level API.
        //   Current plan:
        //     let entity = self.id();
        //     self.queue(move |world| {
        //         EffectFacade::apply_buff(world, entity, buff_id);
        //     });
        self
    }

    fn heal(&mut self, amount: u32) -> &mut Self {
        info!(
            "EntityCommandsExt::heal(amount={}) — queuing heal request",
            amount,
        );
        // TODO[Phase C2]: Wire to ExecutionFacade::heal once the execution
        //   integration facade exposes a command-level API for HP modification.
        //   Current plan:
        //     let entity = self.id();
        //     self.queue(move |world| {
        //         ExecutionFacade::heal(world, entity, amount);
        //     });
        self
    }

    fn kill(&mut self) -> &mut Self {
        info!("EntityCommandsExt::kill() — inserting Dead component");
        self.insert(Dead);
        self
    }
}
