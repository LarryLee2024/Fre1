use bevy::prelude::*;

use super::mechanism::systems::aggregator_system::on_aggregate_dirty;

pub struct AggregatorPlugin;

impl Plugin for AggregatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aggregate_dirty);
    }
}
