//! SkillPanel 组件的类型定义
//!
//! 定义 SkillPanel（面板标记）和 SkillSlotIndex（排序索引）组件。
//! SkillPanel 挂载在容器实体上，SkillSlotIndex 挂载在子技能槽实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// 技能面板容器标记组件
///
/// 标记 Z7 技能面板的容器实体，供清理/查询系统识别。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SkillPanel;

/// 技能槽位排序索引组件
///
/// 标记技能面板下各个技能槽的顺序位置（0-based），
/// 用于按 SkillPanelVm 顺序匹配和 UI 排序。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SkillSlotIndex(pub usize);
