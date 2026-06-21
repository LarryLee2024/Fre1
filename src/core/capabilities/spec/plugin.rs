use bevy::prelude::*;

use crate::core::capabilities::spec::mechanism::SpecRegistry;

/// Spec 领域插件，注册 Spec 注册中心资源与生命周期管理。
pub struct SpecPlugin;

impl Plugin for SpecPlugin {
    fn build(&self, app: &mut App) {
        // Resource: SpecRegistry（Def→Spec 工厂转换）
        app.insert_resource(SpecRegistry::default());

        // Events（Bevy 0.19+ observer-based 事件，无需 add_event）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
