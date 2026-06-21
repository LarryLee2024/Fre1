use bevy::prelude::*;

use super::mechanism::systems::aggregator_system::on_aggregate_dirty;

/// Aggregator 能力插件。
///
/// 负责注册 AggregateDirty 观察者，驱动属性聚合管线的脏标记响应。
pub struct AggregatorPlugin;

impl Plugin for AggregatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aggregate_dirty);
    }
}
