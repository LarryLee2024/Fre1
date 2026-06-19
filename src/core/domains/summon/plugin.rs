//! SummonPlugin — 召唤领域 Plugin
//!
//! 注册召唤组件和系统。
//! 详见 docs/02-domain/domains/summon_domain.md

use bevy::prelude::*;

use super::components::{SummonBond, SummonSlotManager};
use super::resources::SummonConfig;
use super::systems::{on_summon_created, on_summon_expired};

pub struct SummonPlugin;

impl Plugin for SummonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SummonBond>();
        app.register_type::<SummonSlotManager>();

        app.init_resource::<SummonConfig>();

        app.add_observer(on_summon_created);
        app.add_observer(on_summon_expired);
    }
}
