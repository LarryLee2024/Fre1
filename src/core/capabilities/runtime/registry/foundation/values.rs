//! Registry 值对象：DefRegistry 与校验结果

use std::collections::HashMap;

use super::types::{RegistryEntry, RegistryError};

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
            return Err(RegistryError::DuplicateId(def_id));
        }

        // V1: 检查 ID 格式（非空）
        if def_id.is_empty() {
            return Err(RegistryError::InvalidIdFormat(
                "ID must not be empty".into(),
            ));
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
            .ok_or_else(|| RegistryError::IdNotFound(def_id.into()))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_020_registry_empty() {
        let reg = DefRegistry::new();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn unit_021_registry_register() {
        let mut reg = DefRegistry::new();
        let entry = RegistryEntry::new("abl_000001", "Ability", "name=Fireball");
        assert!(reg.register(entry).is_ok());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn unit_022_registry_duplicate_rejected() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
            .unwrap();
        let result = reg.register(RegistryEntry::new("abl_000001", "Ability", ""));
        assert!(result.is_err());
    }

    #[test]
    fn unit_023_registry_get() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", "name=Fireball"))
            .unwrap();
        let entry = reg.get("abl_000001");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().data, "name=Fireball");
    }

    #[test]
    fn unit_024_registry_get_not_found() {
        let reg = DefRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn unit_025_registry_get_by_type() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
            .unwrap();
        reg.register(RegistryEntry::new("abl_000002", "Ability", ""))
            .unwrap();
        reg.register(RegistryEntry::new("eff_000001", "Effect", ""))
            .unwrap();

        let abilities = reg.get_by_type("Ability");
        assert_eq!(abilities.len(), 2);

        let effects = reg.get_by_type("Effect");
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn unit_026_registry_contains() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
            .unwrap();
        assert!(reg.contains("abl_000001"));
        assert!(!reg.contains("abl_999999"));
    }

    #[test]
    fn unit_027_registry_mark_deprecated() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
            .unwrap();
        assert!(
            reg.mark_deprecated("abl_000001", Some("abl_000042".into()))
                .is_ok()
        );

        let entry = reg.get("abl_000001").unwrap();
        assert!(entry.deprecated);
        assert_eq!(entry.superseded_by, Some("abl_000042".into()));
    }

    #[test]
    fn unit_028_registry_all_ids() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
            .unwrap();
        reg.register(RegistryEntry::new("eff_000001", "Effect", ""))
            .unwrap();
        assert_eq!(reg.all_ids().len(), 2);
    }

    #[test]
    fn unit_029_cross_reference_report() {
        let report = CrossReferenceReport::new();
        assert!(!report.has_broken_references());
    }

    #[test]
    fn unit_030_registry_empty_id_rejected() {
        let mut reg = DefRegistry::new();
        let result = reg.register(RegistryEntry::new("", "Ability", ""));
        assert!(result.is_err());
    }
}
