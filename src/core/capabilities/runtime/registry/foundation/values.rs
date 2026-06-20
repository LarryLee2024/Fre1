//! Registry 值对象：DefRegistry 与校验结果

use std::collections::HashMap;

use super::error::RegistryError;
use super::types::RegistryEntry;

/// 全局 Definition 注册中心。
///
/// 所有 Def 在内容加载时通过 Registry 注册，运行时只读。
/// 使用 String → RegistryEntry 映射，不依赖具体 Def 类型。
#[derive(Debug, Clone, PartialEq)]
pub struct DefRegistry {
    /// 所有注册的 Def（def_id → entry）
    entries: HashMap<String, RegistryEntry>,
    /// 按类型索引（def_type → [def_id]）
    type_index: HashMap<String, Vec<String>>,
}

impl DefRegistry {
    /// 创建空的注册中心。
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// 注册一个 Def。
    ///
    /// V2: ID 全局唯一。
    /// # Errors
    /// - DuplicateId: 同 ID 已注册
    pub fn register(&mut self, entry: RegistryEntry) -> Result<(), RegistryError> {
        let def_id = entry.def_id.clone();
        let def_type = entry.def_type.clone();

        if self.entries.contains_key(&def_id) {
            return Err(RegistryError::DuplicateId { id: def_id });
        }

        // V1: 检查 ID 格式（非空）
        if def_id.is_empty() {
            return Err(RegistryError::InvalidIdFormat { id: "ID must not be empty".into() });
        }

        self.entries.insert(def_id.clone(), entry);
        self.type_index.entry(def_type).or_default().push(def_id);

        Ok(())
    }

    /// 按 ID 查询 Def。
    pub fn get(&self, def_id: &str) -> Option<&RegistryEntry> {
        self.entries.get(def_id)
    }

    /// 按类型查询所有 Def。
    pub fn get_by_type(&self, def_type: &str) -> Vec<&RegistryEntry> {
        self.type_index
            .get(def_type)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    /// 检查 ID 是否存在。
    pub fn contains(&self, def_id: &str) -> bool {
        self.entries.contains_key(def_id)
    }

    /// 注册的 Def 总数。
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// 所有已注册的 Def ID。
    pub fn all_ids(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// 获取所有条目。
    pub fn all_entries(&self) -> Vec<&RegistryEntry> {
        self.entries.values().collect()
    }

    /// 标记一个 Def 为已废弃。
    pub fn mark_deprecated(
        &mut self,
        def_id: &str,
        superseded_by: Option<String>,
    ) -> Result<(), RegistryError> {
        let entry = self
            .entries
            .get_mut(def_id)
            .ok_or_else(|| RegistryError::IdNotFound { id: def_id.into() })?;
        entry.deprecated = true;
        entry.superseded_by = superseded_by;
        Ok(())
    }
}

impl Default for DefRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 跨 Def 引用校验报告。
#[derive(Debug, Clone, PartialEq)]
pub struct CrossReferenceReport {
    /// 总 Def 数
    pub total_defs: u32,
    /// 总引用数
    pub total_references: u32,
    /// 断裂引用数
    pub broken_count: u32,
    /// 断裂引用详情
    pub broken_references: Vec<BrokenReference>,
}

impl CrossReferenceReport {
    /// 创建空的引用报告。
    pub fn new() -> Self {
        Self {
            total_defs: 0,
            total_references: 0,
            broken_count: 0,
            broken_references: Vec::new(),
        }
    }

    /// 是否有断裂引用。
    pub fn has_broken_references(&self) -> bool {
        self.broken_count > 0
    }
}

impl Default for CrossReferenceReport {
    fn default() -> Self {
        Self::new()
    }
}

/// 断裂引用详情。
#[derive(Debug, Clone, PartialEq)]
pub struct BrokenReference {
    /// 来源 Def ID
    pub source_def: String,
    /// 引用字段
    pub field: String,
    /// 引用的目标 ID
    pub referenced_id: String,
    /// 期望的类型
    pub expected_type: String,
}
