use bevy::prelude::*;

use crate::core::capabilities::event::mechanism::EventBus;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        // Resource: EventBus（全局事件路由）
        app.insert_resource(EventBus::new());
    }
}
