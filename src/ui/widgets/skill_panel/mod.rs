//! 模块名: SkillPanel Widget — 技能面板有机体
//!
//! 组合多个 SkillSlot 为一个技能面板容器。
//! 每个技能面板包含攻击、火球术、治疗三个技能槽位，
//! 匹配 SkillPanelVm::default() 提供的默认数据。
//!
//! 契约:
//!   输入属性:    无（内部使用默认占位数据，由 SkillPanelVm 投影填充）
//!   输出事件:  SkillSlotAction::Use（由子 SkillSlot 按钮冒泡）
//!   本地状态:      SkillPanel（容器标记）, SkillSlotIndex（排序索引）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{SkillPanel, SkillSlotIndex};

/// SkillPanelPlugin — 注册 SkillPanel Widget 所需的 Component 类型
///
/// 仅注册组件反射类型。SkillPanel 不含独立系统 ——
/// 其子 SkillSlot 的运行由 SkillSlotPlugin 已注册的系统负责。
pub struct SkillPanelPlugin;

impl Plugin for SkillPanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkillPanel>()
            .register_type::<SkillSlotIndex>();
    }
}
