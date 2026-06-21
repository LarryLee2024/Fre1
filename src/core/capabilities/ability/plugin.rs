use bevy::prelude::*;

use crate::core::capabilities::ability::mechanism::AbilityInstanceIdGenerator;

/// Ability 领域插件，注册技能系统的 ECS 资源与生命周期管理。
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityInstanceIdGenerator>();
    }
}
