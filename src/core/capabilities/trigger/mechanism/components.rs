//! Trigger ECS 组件
//!
//! TriggerContainer 挂载在实体上，管理该实体所有已注册的触发器。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::trigger::foundation::{TriggerEntry, TriggerType};

/// 挂载在实体上的触发器容器组件。
///
/// 管理实体的所有触发器实例，支持按 ID 或按类型查询。
/// 触发器注册（register）和移除（remove）流程由外部系统编排。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TriggerContainer {
    /// trigger_id → TriggerEntry
    pub triggers: HashMap<String, TriggerEntry>,
    /// TriggerType → trigger_id 列表（快速按类型查找）
    pub by_type: HashMap<TriggerType, Vec<String>>,
}

impl TriggerContainer {
    /// 创建一个空的 TriggerContainer。triggers 和 by_type 均为初始为空。
    pub fn empty() -> Self {
        Self {
            triggers: HashMap::new(),
            by_type: HashMap::new(),
        }
    }

    /// 注册一个触发器。
    ///
    /// 自动维护 by_type 索引。
    pub fn register(&mut self, entry: TriggerEntry) {
        let entry_type = entry.trigger_type.clone();
        let entry_id = entry.id.clone();
        self.by_type
            .entry(entry_type)
            .or_default()
            .push(entry_id.clone());
        self.triggers.insert(entry_id, entry);
    }

    /// 移除一个触发器。
    ///
    /// 自动清理 by_type 索引。幂等——不存在的 ID 静默忽略。
    pub fn remove(&mut self, trigger_id: &str) {
        if let Some(entry) = self.triggers.remove(trigger_id)
            && let Some(ids) = self.by_type.get_mut(&entry.trigger_type)
        {
            ids.retain(|id| id != trigger_id);
            if ids.is_empty() {
                self.by_type.remove(&entry.trigger_type);
            }
        }
    }

    /// 用于 TriggerSystem 评估期间读取触发器条件。
    pub fn get(&self, trigger_id: &str) -> Option<&TriggerEntry> {
        self.triggers.get(trigger_id)
    }

    /// 用于外部系统修改触发器运行时状态（如禁用/重置频率）。
    pub fn get_mut(&mut self, trigger_id: &str) -> Option<&mut TriggerEntry> {
        self.triggers.get_mut(trigger_id)
    }

    /// 用于 TriggerSystem 在事件发生时精确匹配对应的触发器。
    pub fn find_by_type(&self, trigger_type: &TriggerType) -> Vec<&TriggerEntry> {
        self.by_type
            .get(trigger_type)
            .map(|ids| ids.iter().filter_map(|id| self.triggers.get(id)).collect())
            .unwrap_or_default()
    }

    /// 用于遍历所有触发器（如回合重置、批量检查）。
    pub fn all(&self) -> impl Iterator<Item = &TriggerEntry> {
        self.triggers.values()
    }

    /// 用于外部系统修改触发器状态（如频率计数重置）。
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut TriggerEntry> {
        self.triggers.values_mut()
    }

    /// 触发器数量。
    pub fn len(&self) -> usize {
        self.triggers.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.triggers.is_empty()
    }

    /// 用于 TriggerSystem 清空后重建。
    pub fn clear(&mut self) {
        self.triggers.clear();
        self.by_type.clear();
    }

    /// 回合结束时重置所有触发器的触发计数。
    pub fn reset_turn_counts(&mut self) {
        for entry in self.triggers.values_mut() {
            entry.reset_turn_count();
        }
    }

    /// 获取匹配指定触发类型且当前允许触发的触发器列表。
    pub fn find_ready(&self, trigger_type: &TriggerType) -> Vec<&TriggerEntry> {
        self.find_by_type(trigger_type)
            .into_iter()
            .filter(|entry| entry.can_trigger())
            .collect()
    }
}

impl Default for TriggerContainer {
    fn default() -> Self {
        Self::empty()
    }
}
