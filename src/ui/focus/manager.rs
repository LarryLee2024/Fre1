//! FocusManager — 全局焦点状态管理
//!
//! 跟踪当前活跃焦点组和聚焦元素，支持焦点历史记录。
//! 当 Overlay（如 Modal）弹出时记录当前焦点，关闭时恢复。
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §2.1

use std::collections::HashMap;

use bevy::prelude::*;

/// 焦点管理器 — 跟踪当前焦点组和聚焦元素
///
/// # 生命周期
/// - Screen 激活时，其根 FocusGroup 获得活跃
/// - FocusSystem 从组内 priority 最高的 Focusable 开始
/// - Overlay 弹出时记录当前焦点到 focus_history
/// - Overlay 关闭时从 focus_history 恢复
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct FocusManager {
    /// 当前活跃的焦点组 ID
    pub active_group: Option<u32>,
    /// 当前聚焦的实体
    pub focused_entity: Option<Entity>,
    /// 每个焦点组内的索引位置 (group_id → 组内索引)
    pub group_indices: HashMap<u32, usize>,
    /// 焦点历史（用于 Overlay 关闭后恢复）
    /// 每组记录退出前的聚焦元素
    pub focus_history: HashMap<u32, Entity>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self {
            active_group: None,
            focused_entity: None,
            group_indices: HashMap::new(),
            focus_history: HashMap::new(),
        }
    }
}

impl FocusManager {
    /// 聚焦到指定实体
    ///
    /// 将指定实体设为当前焦点，同时更新其所在组的索引。
    pub fn focus(&mut self, entity: Entity, group_id: u32, index: usize) {
        self.focused_entity = Some(entity);
        self.active_group = Some(group_id);
        self.group_indices.insert(group_id, index);
    }

    /// 清除当前焦点
    ///
    /// 清空聚焦实体和活跃组，焦点视觉系统将移除所有高亮。
    pub fn blur(&mut self) {
        self.focused_entity = None;
        self.active_group = None;
    }

    /// 记录当前焦点到历史（Overlay 弹出时调用）
    ///
    /// 保存当前组和聚焦元素，以便 Overlay 关闭后恢复。
    pub fn push_focus(&mut self) {
        if let (Some(group), Some(entity)) = (self.active_group, self.focused_entity) {
            self.focus_history.insert(group, entity);
        }
    }

    /// 从历史恢复焦点（Overlay 关闭时调用）
    ///
    /// 如果该组有历史记录，恢复上次聚焦的元素。
    /// 返回 true 表示成功恢复，false 表示无历史记录。
    pub fn pop_focus(&mut self, group_id: u32) -> Option<Entity> {
        let entity = self.focus_history.remove(&group_id)?;
        self.focused_entity = Some(entity);
        self.active_group = Some(group_id);
        Some(entity)
    }

    /// 激活指定焦点组
    ///
    /// 设置活跃组，如果该组有历史记录则恢复焦点。
    pub fn activate_group(&mut self, group_id: u32, first_entity: Option<(Entity, usize)>) {
        self.active_group = Some(group_id);

        // 优先恢复历史焦点
        if let Some(&entity) = self.focus_history.get(&group_id) {
            self.focused_entity = Some(entity);
            return;
        }

        // 无历史记录时使用传入的第一个实体
        if let Some((entity, index)) = first_entity {
            self.focused_entity = Some(entity);
            self.group_indices.insert(group_id, index);
        }
    }

    /// 检查指定实体是否为当前焦点
    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused_entity == Some(entity)
    }
}
