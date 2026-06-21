//! 标签生命周期管理 — TagHierarchy Resource 与注册/验证

use std::collections::{HashMap, HashSet};

use crate::core::capabilities::tag::events::TagHierarchyChanged;
use crate::core::capabilities::tag::foundation::{BitMask, TagDefinition, TagId};
use bevy::prelude::*;

/// 全局标签层级树（Resource）。
///
/// 在内容加载阶段构建，运行时只读。
#[derive(Resource, Debug, Clone, Default)]
pub struct TagHierarchy {
    /// 所有已注册标签的完整映射
    pub tags: HashMap<TagId, TagDefinition>,
    /// 子标签索引: parent_id → Vec<child_id>
    pub children: HashMap<TagId, Vec<TagId>>,
    /// 位掩码继承映射: tag_id → 包含自身及所有子标签的位掩码
    pub inherited_masks: HashMap<TagId, BitMask>,
    /// 所有抽象标签集合
    pub abstract_tags: HashSet<TagId>,
}

/// 标签注册错误
#[derive(Debug)]
pub enum TagRegistrationError {
    DuplicateId(TagId),
    ParentNotFound(TagId),
    CircularDependency(TagId),
    NamespaceMismatch { child: TagId, parent: TagId },
}

impl std::fmt::Display for TagRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate tag ID: {}", id),
            Self::ParentNotFound(id) => write!(f, "parent tag not found: {}", id),
            Self::CircularDependency(id) => {
                write!(f, "circular dependency detected for tag: {}", id)
            }
            Self::NamespaceMismatch { child, parent } => {
                write!(
                    f,
                    "namespace mismatch: child {} has different namespace from parent {}",
                    child, parent
                )
            }
        }
    }
}

impl std::error::Error for TagRegistrationError {}

impl TagHierarchy {
    /// 注册单个标签，执行所有校验。
    pub fn register(
        &mut self,
        def: TagDefinition,
        commands: &mut Commands,
    ) -> Result<(), TagRegistrationError> {
        // V1: TagId 全局唯一
        if self.tags.contains_key(&def.id) {
            return Err(TagRegistrationError::DuplicateId(def.id));
        }

        // V3: 如果有父标签，父标签必须已注册
        if let Some(ref parent_id) = def.parent_id {
            if !self.tags.contains_key(parent_id) {
                return Err(TagRegistrationError::ParentNotFound(def.id));
            }

            // V4: 命名空间一致性
            let parent = &self.tags[parent_id];
            if parent.namespace != def.namespace {
                return Err(TagRegistrationError::NamespaceMismatch {
                    child: def.id.clone(),
                    parent: parent_id.clone(),
                });
            }

            // V2: 无循环层级（DFS）
            if self.would_create_cycle(&def.id, parent_id) {
                return Err(TagRegistrationError::CircularDependency(def.id));
            }
        }

        // 标签注册核心逻辑：存储定义 + 维护父子索引
        let id = def.id.clone();
        let bit_index = def.bit_index;

        // 捕获事件所需数据（def 即将被 move）
        let id_for_event = id.clone();
        let parent_id_for_event = def.parent_id.clone();

        if def.is_abstract {
            self.abstract_tags.insert(id.clone());
        }

        // children 索引用于高效查询子标签，避免每次遍历全部标签
        if let Some(ref parent_id) = def.parent_id {
            self.children
                .entry(parent_id.clone())
                .or_default()
                .push(id.clone());
        }

        self.tags.insert(id, def);

        // 重建继承掩码
        self.rebuild_inherited_masks(bit_index);

        commands.trigger(TagHierarchyChanged {
            parent_tag_id: parent_id_for_event.unwrap_or_else(|| id_for_event.clone()),
            affected_child_ids: vec![id_for_event],
        });

        Ok(())
    }

    /// 批量注册标签（两阶段加载：先注册所有节点，再构建层级关系）。
    pub fn register_batch(
        &mut self,
        defs: Vec<TagDefinition>,
        commands: &mut Commands,
    ) -> Vec<Result<(), TagRegistrationError>> {
        let results: Vec<_> = defs
            .into_iter()
            .map(|def| self.register(def, commands))
            .collect();
        results
    }

    /// 检查是否会形成循环引用
    fn would_create_cycle(&self, child_id: &TagId, parent_id: &TagId) -> bool {
        let mut visited = HashSet::new();
        let mut current = parent_id.clone();
        loop {
            if &current == child_id {
                return true;
            }
            if !visited.insert(current.clone()) {
                return false; // already visited, no cycle on this path
            }
            match self
                .tags
                .get(&current)
                .and_then(|def| def.parent_id.clone())
            {
                Some(next) => current = next,
                None => return false,
            }
        }
    }

    /// 重建从指定 bit_index 开始的继承掩码
    fn rebuild_inherited_masks(&mut self, _new_bit_index: u32) {
        // 完整重建: 遍历所有标签，构建包含子标签的位掩码
        let all_ids: Vec<TagId> = self.tags.keys().cloned().collect();

        for id in &all_ids {
            let mut mask: BitMask = 1 << self.tags[id].bit_index;
            mask |= self.collect_child_bits(id);
            self.inherited_masks.insert(id.clone(), mask);
        }
    }

    /// 递归收集所有子标签的位
    fn collect_child_bits(&self, parent_id: &TagId) -> BitMask {
        let mut mask: BitMask = 0;
        if let Some(children) = self.children.get(parent_id) {
            for child_id in children {
                if let Some(def) = self.tags.get(child_id) {
                    mask |= 1 << def.bit_index;
                    mask |= self.collect_child_bits(child_id);
                }
            }
        }
        mask
    }

    /// 获取标签的继承掩码（包含自身及所有子标签）
    pub fn inherited_mask(&self, tag_id: &TagId) -> BitMask {
        self.inherited_masks.get(tag_id).copied().unwrap_or(0)
    }
}
