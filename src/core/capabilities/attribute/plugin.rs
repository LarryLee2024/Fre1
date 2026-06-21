use bevy::prelude::*;

use super::content::register_attributes_from_content;
use super::mechanism::AttributeRegistry;
use super::mechanism::systems::attribute_system::on_attribute_initialized;
use crate::content::LoadedAttributeDefs;

/// Attribute 能力插件。
///
/// 负责初始化属性注册表、注册内容加载回调和属性初始化观察者。
pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeRegistry>();
        app.add_observer(on_attribute_initialized);
        app.add_systems(
            Update,
            register_attributes_from_content.run_if(resource_changed::<LoadedAttributeDefs>),
        );
    }
}
