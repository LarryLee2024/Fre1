use bevy::prelude::*;

use super::mechanism::systems::condition_system::{
    on_attribute_changed, on_tag_changed_by_tag_added, on_tag_changed_by_tag_removed,
};

/// Condition 能力插件——注册条件依赖变更监听。
///
/// 观察 Tag/Attribute 变更，自动标记依赖该状态的条件为待重新评估。
pub struct ConditionPlugin;

impl Plugin for ConditionPlugin {
    fn build(&self, app: &mut App) {
        // 观察标签变更 → 标记依赖该标签的条件为 dirty
        app.add_observer(on_tag_changed_by_tag_added);
        app.add_observer(on_tag_changed_by_tag_removed);

        // 观察属性变更 → 标记依赖该属性的条件为 dirty
        app.add_observer(on_attribute_changed);
    }
}
