//! Condition ECS 组件
//!
//! ConditionContainer 记录实体上待评估/失败的条件状态，
//! 支持条件订阅（等待相关状态变化后重评估）。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::condition::foundation::{Condition, ConditionResult};

/// 单条条件的跟踪状态。
#[derive(Debug, Clone)]
pub struct ConditionEntry {
    /// 条件定义
    pub condition: Condition,
    /// 上次评估结果
    pub last_result: Option<ConditionResult>,
    /// 该条件依赖的标签/属性 ID 列表（用于订阅通知）
    pub dependencies: Vec<String>,
}

/// 挂载在实体上的条件容器组件。
///
/// 存储实体的待评估/已评估条件和订阅信息。
/// 当实体状态变化（Tag/Attribute 变更）时，相关条件被标记为
/// "待重新评估"（last_result 设为 None）。
#[derive(Component, Debug, Clone)]
pub struct ConditionContainer {
    /// 条件 ID → 条件条目
    pub conditions: HashMap<String, ConditionEntry>,
    /// 该实体当前依赖的标签/属性 ID → 关联的条件 ID 列表
    pub tag_dependents: HashMap<String, Vec<String>>,
    pub attribute_dependents: HashMap<String, Vec<String>>,
}

impl ConditionContainer {
    /// 创建一个空的容器。
    pub fn empty() -> Self {
        Self {
            conditions: HashMap::new(),
            tag_dependents: HashMap::new(),
            attribute_dependents: HashMap::new(),
        }
    }

    /// 注册一个条件到容器中。
    ///
    /// 自动提取条件依赖的 tag/attribute ID 建立反向索引。
    pub fn add_condition(&mut self, id: impl Into<String>, condition: Condition) {
        let id = id.into();
        let deps = Self::extract_dependencies(&condition);

        for tag_id in &deps.tag_ids {
            self.tag_dependents
                .entry(tag_id.clone())
                .or_default()
                .push(id.clone());
        }
        for attr_id in &deps.attribute_ids {
            self.attribute_dependents
                .entry(attr_id.clone())
                .or_default()
                .push(id.clone());
        }

        self.conditions.insert(
            id,
            ConditionEntry {
                condition,
                last_result: None,
                dependencies: deps.all_ids(),
            },
        );
    }

    /// 当标签变更时，标记依赖该标签的条件为待重新评估。
    pub fn on_tag_changed(&mut self, tag_id: &str) {
        if let Some(condition_ids) = self.tag_dependents.get(tag_id) {
            for cid in condition_ids {
                if let Some(entry) = self.conditions.get_mut(cid) {
                    entry.last_result = None;
                }
            }
        }
    }

    /// 当属性变更时，标记依赖该属性的条件为待重新评估。
    pub fn on_attribute_changed(&mut self, attribute_id: &str) {
        if let Some(condition_ids) = self.attribute_dependents.get(attribute_id) {
            for cid in condition_ids {
                if let Some(entry) = self.conditions.get_mut(cid) {
                    entry.last_result = None;
                }
            }
        }
    }

    /// 提取条件树中所有依赖的标签和属性 ID。
    fn extract_dependencies(condition: &Condition) -> DependencyInfo {
        let mut info = DependencyInfo::default();
        Self::collect_deps(condition, &mut info);
        info
    }

    fn collect_deps(condition: &Condition, info: &mut DependencyInfo) {
        match condition {
            Condition::TagRequirement { tag_id, .. } => {
                info.tag_ids.push(tag_id.clone());
            }
            Condition::AttributeCheck { attribute_id, .. } => {
                info.attribute_ids.push(attribute_id.clone());
            }
            Condition::ResourceCheck { resource_id, .. } => {
                info.attribute_ids.push(resource_id.clone());
            }
            Condition::And(children) | Condition::Or(children) => {
                for child in children {
                    Self::collect_deps(child, info);
                }
            }
            Condition::Not(child) => {
                Self::collect_deps(child, info);
            }
            Condition::Custom(_) => {
                // 自定义条件的依赖无法静态提取，由外部绑定
            }
        }
    }

    /// 获取指定条件 ID 的条目。
    pub fn get(&self, id: &str) -> Option<&ConditionEntry> {
        self.conditions.get(id)
    }

    /// 获取指定条件 ID 的可变引用。
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ConditionEntry> {
        self.conditions.get_mut(id)
    }

    /// 标记所有条件为待重新评估。
    pub fn invalidate_all(&mut self) {
        for entry in self.conditions.values_mut() {
            entry.last_result = None;
        }
    }

    /// 移除一个条件及其依赖追踪。
    pub fn remove(&mut self, id: &str) {
        self.conditions.remove(id);
        self.tag_dependents.retain(|_, ids| {
            ids.retain(|cid| cid != id);
            !ids.is_empty()
        });
        self.attribute_dependents.retain(|_, ids| {
            ids.retain(|cid| cid != id);
            !ids.is_empty()
        });
    }
}

impl Default for ConditionContainer {
    fn default() -> Self {
        Self::empty()
    }
}

/// 依赖信息（内部辅助）。
#[derive(Default)]
struct DependencyInfo {
    tag_ids: Vec<String>,
    attribute_ids: Vec<String>,
}

impl DependencyInfo {
    fn all_ids(&self) -> Vec<String> {
        let mut all = Vec::with_capacity(self.tag_ids.len() + self.attribute_ids.len());
        all.extend(self.tag_ids.iter().cloned());
        all.extend(self.attribute_ids.iter().cloned());
        all
    }
}
