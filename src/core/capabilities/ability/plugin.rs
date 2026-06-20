use bevy::prelude::*;

use crate::core::capabilities::ability::mechanism::AbilityInstanceIdGenerator;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityInstanceIdGenerator>();
    }
}
