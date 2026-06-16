use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::AttributeId;

#[derive(Event, Debug, Clone)]
pub struct AttributeChanged {
    pub entity: Entity,
    pub attribute_id: AttributeId,
    pub old_value: f32,
    pub new_value: f32,
}

#[derive(Event, Debug, Clone)]
pub struct AttributeInitialized {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct AttributeClamped {
    pub entity: Entity,
    pub attribute_id: AttributeId,
    pub attempted_value: f32,
    pub clamped_value: f32,
}
