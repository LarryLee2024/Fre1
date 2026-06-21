//! Module Name: SkillSlot Widget — 技能槽复合控件
//!
//! 组合 Panel / Text / ProgressBar / Button 四个原子组件为一个技能槽卡片。
//! 每个技能槽显示一个技能的名称、冷却进度和使用按钮。
//!
//! Contract:
//!   Props (input):    name, cooldown_max（通过 SkillSlotState）
//!   Events (output):  SkillSlotAction::Use 标记在按钮实体上供 Observer 路由
//!   Local State:      SkillSlotState（name, cooldown_current, cooldown_max, is_ready）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{SkillSlotAction, SkillSlotNameLabel, SkillSlotState};
use self::systems::{refresh_skill_slot_from_vm, skill_slot_update_system};

/// SkillSlotPlugin — 注册 SkillSlot Widget 所需的 Component/System
pub struct SkillSlotPlugin;

impl Plugin for SkillSlotPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkillSlotState>()
            .register_type::<SkillSlotAction>()
            .register_type::<SkillSlotNameLabel>()
            .add_systems(
                Update,
                (skill_slot_update_system, refresh_skill_slot_from_vm),
            );
    }
}
