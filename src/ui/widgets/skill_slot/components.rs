//! SkillSlot 组件的类型定义
//!
//! 定义 SkillSlotState（Widget Contract 的本地状态）和 SkillSlotAction（按钮动作标记）。
//! SkillSlotState 挂载在容器实体上，SkillSlotAction 挂载在按钮实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// 技能槽本地状态（Widget Contract Local State）
///
/// 包含技能名称、冷却时间最大值、当前冷却时间和就绪状态。
/// Props 字段由 spawn_skill_slot 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SkillSlotState {
    /// 技能显示名称
    pub name: String,
    /// 技能定义 ID（用于关联 SkillPanelVm.skills 中的条目）
    pub skill_id: u32,
    /// 冷却时间最大值（帧数或秒数，由系统约定）
    pub cooldown_max: u32,
    /// 当前冷却时间（递减至 0 表示就绪）
    pub cooldown_current: u32,
    /// 技能是否就绪（cooldown_current == 0）
    pub is_ready: bool,
}

/// 技能槽按钮动作标记
///
/// 标记技能槽内的按钮为"使用技能"动作，供 Observer 或其他系统
/// 识别技能槽的交互意图。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SkillSlotAction {
    /// 使用技能
    Use,
}

/// SkillSlot 名称文本标记组件
///
/// 标记 SkillSlot 下用于显示技能名称的 Text 子实体，
/// 供 refresh_skill_slot_from_vm 系统查询和更新。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SkillSlotNameLabel;
