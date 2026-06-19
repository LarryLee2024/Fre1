use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::modifier::foundation::{ModifierData, ModifierInstanceId};

/// 实体上的活跃修改器容器。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ModifierContainer {
    pub modifiers: HashMap<String, Vec<ModifierData>>,
    pub override_index: HashMap<String, ModifierInstanceId>,
    pub max_modifiers: u32,
}

impl ModifierContainer {
    pub fn empty() -> Self {
        Self {
            modifiers: HashMap::new(),
            override_index: HashMap::new(),
            max_modifiers: 100,
        }
    }
}

impl Default for ModifierContainer {
    fn default() -> Self {
        Self::empty()
    }
}
