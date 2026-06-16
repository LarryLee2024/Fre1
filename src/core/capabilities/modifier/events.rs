use bevy::prelude::*;

use crate::core::capabilities::modifier::foundation::ModifierData;

#[derive(Event, Debug, Clone)]
pub struct ModifierApplied {
    pub entity: Entity,
    pub modifier_data: ModifierData,
}

#[derive(Event, Debug, Clone)]
pub struct ModifierRemoved {
    pub entity: Entity,
    pub modifier_data: ModifierData,
}
