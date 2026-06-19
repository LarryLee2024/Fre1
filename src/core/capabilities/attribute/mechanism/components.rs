use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::{AttributeId, AttributeValue};

/// 实体上的属性容器。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AttributeContainer {
    pub attributes: HashMap<AttributeId, AttributeValue>,
    pub derived_cache: HashMap<AttributeId, f32>,
}

impl AttributeContainer {
    pub fn empty() -> Self {
        Self {
            attributes: HashMap::new(),
            derived_cache: HashMap::new(),
        }
    }
}

impl Default for AttributeContainer {
    fn default() -> Self {
        Self::empty()
    }
}
