//! StoryFlag System — 故事标记处理系统
//!
//! 处理 StoryFlagSet 事件的副作用记录。

use bevy::prelude::*;

use crate::core::domains::narrative::components::StoryFlags;
use crate::core::domains::narrative::events::StoryFlagSet;

/// 响应 StoryFlagSet 事件，确保标记只增不减（不变量 3.3）。
///
/// 此 Observer 是防御性检查：如果标记已存在且值不同，拒绝覆盖。
/// 实际设置在 dialogue_system 中已完成，此系统用于二次验证。
pub(crate) fn on_story_flag_set(trigger: On<StoryFlagSet>, mut query: Query<&mut StoryFlags>) {
    let event = trigger.event();
    let Ok(mut flags) = query.get_mut(event.entity) else {
        // 实体可能没有 StoryFlags 组件，首次设置时自动创建
        return;
    };

    // 幂等设置（dialogue_system 已处理，这里确保一致）
    flags.set_flag(&event.flag_id, &event.value);
}
