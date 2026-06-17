//! Registry 核心类型
//!
//! Definition 注册中心的核心数据结构：提供通用的类型化存储桶、
//! 变更追踪、查询索引以及完整的 DefinitionRegistry Resource。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md

use bevy::prelude::*;
use std::collections::HashMap;

// ============================================================================
// DefinitionId
// ============================================================================

/// Definition 标识符。
///
/// 格式: `{prefix}_{digits}`（如 `abl_000001`, `eff_000042`）
/// 由 IdAllocator 生成，也可从配置直接指定。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefinitionId(pub String);

impl DefinitionId {
    /// 从字符串创建 DefinitionId。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 返回内部字符串引用。
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 消耗自身，返回内部字符串。
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for DefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DefinitionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DefinitionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for DefinitionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// RegistryEntry
// ============================================================================

/// 通用 Definition 条目（v1 直接值存储）。
///
/// 当具体 Def 类型尚未定义时使用，存储为 JSON 值。
/// 各领域定型后应迁移到 `RegistryBucket<ConcreteDef>` 形式。
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// Def ID
    pub id: DefinitionId,
    /// 序列化的 Def 数据
    pub data: Option<serde_json::Value>,
    /// 是否为废弃状态
    pub deprecated: bool,
    /// 如果被取代，指向新的 ID
    pub superseded_by: Option<DefinitionId>,
}

impl RegistryEntry {
    /// 创建新的 RegistryEntry。
    pub fn new(id: impl Into<DefinitionId>) -> Self {
        Self {
            id: id.into(),
            data: None,
            deprecated: false,
            superseded_by: None,
        }
    }

    /// 设置序列化数据。
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// 标记为废弃。
    pub fn mark_deprecated(&mut self) {
        self.deprecated = true;
    }

    /// 设置取代者。
    pub fn supersede(&mut self, new_id: impl Into<DefinitionId>) {
        self.superseded_by = Some(new_id.into());
    }
}

// ============================================================================
// IndexKey
// ============================================================================

/// 查询索引键。
///
/// 用于按 tag/category/namespace 等分类索引检索 DefinitionId 列表。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexKey {
    /// 索引分类名称（如 "category", "movement_type", "damage_type"）
    pub category: String,
    /// 索引值
    pub value: String,
}

impl IndexKey {
    /// 创建索引键。
    pub fn new(category: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            value: value.into(),
        }
    }
}

// ============================================================================
// RegistryBucket<T>
// ============================================================================

/// 类型安全的 Definition 存储桶。
///
/// 提供版本化的 `DefinitionId → T` 映射，支持查询、索引和变更追踪。
/// 每个 Def 类型对应一个专用桶。
#[derive(Debug, Clone)]
pub struct RegistryBucket<T> {
    /// DefinitionId → 数据的核心映射
    items: HashMap<DefinitionId, T>,
    /// 分类索引
    indices: HashMap<IndexKey, Vec<DefinitionId>>,
    /// 变更版本号（每次写入递增）
    version: u64,
}

impl<T> RegistryBucket<T> {
    /// 创建空的存储桶。
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            indices: HashMap::new(),
            version: 0,
        }
    }

    /// 通过 DefinitionId 查询条目。
    pub fn get(&self, id: &DefinitionId) -> Option<&T> {
        self.items.get(id)
    }

    /// 通过字符串 ID 查询条目（便利方法）。
    pub fn get_str(&self, id: &str) -> Option<&T> {
        self.items.keys().find_map(|k| {
            if k.as_str() == id {
                self.items.get(k)
            } else {
                None
            }
        })
    }

    /// 可变引用查询。
    pub fn get_mut(&mut self, id: &DefinitionId) -> Option<&mut T> {
        self.items.get_mut(id)
    }

    /// 插入条目。返回被替换的旧值（如有）。
    pub fn insert(&mut self, id: impl Into<DefinitionId>, value: T) -> Option<T> {
        let id = id.into();
        let old = self.items.insert(id, value);
        self.version += 1;
        old
    }

    /// 移除条目。返回被移除的值。
    pub fn remove(&mut self, id: &DefinitionId) -> Option<T> {
        let value = self.items.remove(id);
        if value.is_some() {
            self.version += 1;
        }
        value
    }

    /// 检查 ID 是否存在。
    pub fn contains(&self, id: &DefinitionId) -> bool {
        self.items.contains_key(id)
    }

    /// 遍历所有条目。
    pub fn iter(&self) -> impl Iterator<Item = (&DefinitionId, &T)> {
        self.items.iter()
    }

    /// 条目数量。
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 当前桶的变更版本号。
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 返回所有 DefinitionId 的副本。
    pub fn ids(&self) -> Vec<DefinitionId> {
        self.items.keys().cloned().collect()
    }

    /// 遍历所有键。
    pub fn keys(&self) -> impl Iterator<Item = &DefinitionId> {
        self.items.keys()
    }

    /// 遍历所有值。
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.items.values()
    }

    /// 清空桶。
    pub fn clear(&mut self) {
        self.items.clear();
        self.indices.clear();
        self.version += 1;
    }

    /// 添加索引条目。
    pub fn add_index(&mut self, key: IndexKey, id: DefinitionId) {
        self.indices.entry(key).or_default().push(id);
    }

    /// 按索引查询 DefinitionId 列表。
    pub fn query_index(&self, key: &IndexKey) -> Vec<DefinitionId> {
        self.indices.get(key).cloned().unwrap_or_default()
    }

    /// 返回所有索引键。
    pub fn index_keys(&self) -> impl Iterator<Item = &IndexKey> {
        self.indices.keys()
    }
}

impl<T> Default for RegistryBucket<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DefinitionRegistry Resource
// ============================================================================

/// 全局 Definition 注册中心 Resource。
///
/// 管理所有游戏定义数据的类型安全存储桶。
/// 属于 Definition 层，由 RegistryPlugin 初始化。
///
/// 当前使用 `RegistryEntry`（v1 直接值）存储。
/// 各领域 Def 类型定型后，逐步迁移到 `RegistryBucket<ConcreteDef>`。
#[derive(Resource)]
pub struct DefinitionRegistry {
    // ---- Capabilities (10) ----
    pub abilities: RegistryBucket<RegistryEntry>,
    pub effects: RegistryBucket<RegistryEntry>,
    pub modifiers: RegistryBucket<RegistryEntry>,
    pub tags: RegistryBucket<RegistryEntry>,
    pub attributes: RegistryBucket<RegistryEntry>,
    pub triggers: RegistryBucket<RegistryEntry>,
    pub cues: RegistryBucket<RegistryEntry>,
    pub items: RegistryBucket<RegistryEntry>,
    pub spells: RegistryBucket<RegistryEntry>,
    pub buffs: RegistryBucket<RegistryEntry>,

    // ---- Domains (5) ----
    pub factions: RegistryBucket<RegistryEntry>,
    pub terrains: RegistryBucket<RegistryEntry>,
    pub recipes: RegistryBucket<RegistryEntry>,
    pub loot_tables: RegistryBucket<RegistryEntry>,
    pub quests: RegistryBucket<RegistryEntry>,

    /// 自定义扩展桶（Domain 可注册额外的 Def 类型）
    pub custom: RegistryBucket<RegistryEntry>,

    /// 上次变更的桶名称（用于事件通知）
    last_changed_bucket: Option<&'static str>,
}

impl DefinitionRegistry {
    /// 创建空的 DefinitionRegistry。
    pub fn new() -> Self {
        Self {
            abilities: RegistryBucket::new(),
            effects: RegistryBucket::new(),
            modifiers: RegistryBucket::new(),
            tags: RegistryBucket::new(),
            attributes: RegistryBucket::new(),
            triggers: RegistryBucket::new(),
            cues: RegistryBucket::new(),
            items: RegistryBucket::new(),
            spells: RegistryBucket::new(),
            buffs: RegistryBucket::new(),
            factions: RegistryBucket::new(),
            terrains: RegistryBucket::new(),
            recipes: RegistryBucket::new(),
            loot_tables: RegistryBucket::new(),
            quests: RegistryBucket::new(),
            custom: RegistryBucket::new(),
            last_changed_bucket: None,
        }
    }

    /// 按名称获取桶的可变引用（用于泛型操作）。
    pub fn bucket_mut(&mut self, name: &str) -> Option<&mut RegistryBucket<RegistryEntry>> {
        match name {
            "abilities" => Some(&mut self.abilities),
            "effects" => Some(&mut self.effects),
            "modifiers" => Some(&mut self.modifiers),
            "tags" => Some(&mut self.tags),
            "attributes" => Some(&mut self.attributes),
            "triggers" => Some(&mut self.triggers),
            "cues" => Some(&mut self.cues),
            "items" => Some(&mut self.items),
            "spells" => Some(&mut self.spells),
            "buffs" => Some(&mut self.buffs),
            "factions" => Some(&mut self.factions),
            "terrains" => Some(&mut self.terrains),
            "recipes" => Some(&mut self.recipes),
            "loot_tables" => Some(&mut self.loot_tables),
            "quests" => Some(&mut self.quests),
            "custom" => Some(&mut self.custom),
            _ => None,
        }
    }

    /// 按名称获取桶的只读引用。
    pub fn bucket(&self, name: &str) -> Option<&RegistryBucket<RegistryEntry>> {
        match name {
            "abilities" => Some(&self.abilities),
            "effects" => Some(&self.effects),
            "modifiers" => Some(&self.modifiers),
            "tags" => Some(&self.tags),
            "attributes" => Some(&self.attributes),
            "triggers" => Some(&self.triggers),
            "cues" => Some(&self.cues),
            "items" => Some(&self.items),
            "spells" => Some(&self.spells),
            "buffs" => Some(&self.buffs),
            "factions" => Some(&self.factions),
            "terrains" => Some(&self.terrains),
            "recipes" => Some(&self.recipes),
            "loot_tables" => Some(&self.loot_tables),
            "quests" => Some(&self.quests),
            "custom" => Some(&self.custom),
            _ => None,
        }
    }

    /// 返回所有桶名称的迭代器。
    pub fn bucket_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "abilities",
            "effects",
            "modifiers",
            "tags",
            "attributes",
            "triggers",
            "cues",
            "items",
            "spells",
            "buffs",
            "factions",
            "terrains",
            "recipes",
            "loot_tables",
            "quests",
            "custom",
        ]
        .into_iter()
    }

    /// 记录最近变更的桶（由插入操作自动调用）。
    pub fn mark_changed(&mut self, bucket_name: &'static str) {
        self.last_changed_bucket = Some(bucket_name);
    }

    /// 消费并返回最近变更的桶名称。
    pub fn take_changed(&mut self) -> Option<&'static str> {
        self.last_changed_bucket.take()
    }

    /// 检查所有桶是否都为空。
    pub fn is_empty(&self) -> bool {
        self.bucket_names()
            .all(|name| self.bucket(name).map(|b| b.is_empty()).unwrap_or(true))
    }

    /// 返回注册的 Def 总数。
    pub fn total_count(&self) -> usize {
        self.bucket_names()
            .filter_map(|name| self.bucket(name))
            .map(|b| b.len())
            .sum()
    }
}

impl Default for DefinitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DefinitionType Trait
// ============================================================================

/// Definition 类型 trait。
///
/// 每个 Def 数据类应实现此 trait，提供从配置创建和校验的能力。
/// 当各领域的 Def 类型定型后，由对应 struct 实现。
///
/// 当前阶段为注册中心的基础设施提供，具体实现随领域推进。
pub trait DefinitionType: Sized {
    /// 配置反序列化类型
    type Config: serde::de::DeserializeOwned;

    /// 桶名称（与 DefinitionRegistry 字段对应）
    const BUCKET_NAME: &'static str;

    /// 配置文件扩展名
    const EXTENSION: &'static str;

    /// 从配置创建 Def 实例。
    fn from_config(config: Self::Config) -> Result<Self, String>;

    /// 注册后校验（ID 格式、引用完整性）。
    fn validate(&self, _registry: &DefinitionRegistry) -> Result<(), Vec<String>> {
        Ok(())
    }
}

// ============================================================================
// Events
// ============================================================================

/// Definition 热重载完成事件。
///
/// 当某桶的 Asset 被重新加载时触发，下游 Observer 可响应刷新。
#[derive(Event)]
pub struct OnDefinitionReloaded {
    /// 发生变更的桶名称（如 "abilities", "effects"）
    pub bucket_name: &'static str,
    /// 变更后的版本号
    pub new_version: u64,
    /// 发生变更的 DefinitionId 列表
    pub changed_ids: Vec<DefinitionId>,
}

impl OnDefinitionReloaded {
    /// 创建热重载事件。
    pub fn new(
        bucket_name: &'static str,
        new_version: u64,
        changed_ids: Vec<DefinitionId>,
    ) -> Self {
        Self {
            bucket_name,
            new_version,
            changed_ids,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_insert_and_get() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        let id = DefinitionId::new("abl_000001");
        let entry = RegistryEntry::new(id.clone());

        assert!(bucket.insert(id.clone(), entry).is_none());
        assert!(bucket.contains(&id));
        assert_eq!(bucket.len(), 1);
        assert_eq!(bucket.version(), 1);
    }

    #[test]
    fn test_bucket_replace() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        let id = DefinitionId::new("abl_000001");
        let old_entry =
            RegistryEntry::new(id.clone()).with_data(serde_json::json!({"name": "old"}));
        let new_entry =
            RegistryEntry::new(id.clone()).with_data(serde_json::json!({"name": "new"}));

        bucket.insert(id.clone(), old_entry);
        let replaced = bucket.insert(id.clone(), new_entry);
        assert!(replaced.is_some());
        assert_eq!(bucket.len(), 1);
    }

    #[test]
    fn test_bucket_remove() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        let id = DefinitionId::new("eff_000001");
        bucket.insert(id.clone(), RegistryEntry::new(id.clone()));

        assert_eq!(bucket.version(), 1);
        let removed = bucket.remove(&id);
        assert!(removed.is_some());
        assert!(bucket.is_empty());
        assert_eq!(bucket.version(), 2);
    }

    #[test]
    fn test_bucket_iter_and_ids() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        let id1 = DefinitionId::new("abl_000001");
        let id2 = DefinitionId::new("abl_000002");
        bucket.insert(id1.clone(), RegistryEntry::new(id1.clone()));
        bucket.insert(id2.clone(), RegistryEntry::new(id2.clone()));

        let ids = bucket.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        let count = bucket.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_bucket_index() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        let id = DefinitionId::new("abl_000001");
        bucket.insert(id.clone(), RegistryEntry::new(id.clone()));

        let key = IndexKey::new("category", "active");
        bucket.add_index(key.clone(), id.clone());

        let results = bucket.query_index(&key);
        assert_eq!(results, vec![id]);
    }

    #[test]
    fn test_definition_registry_new() {
        let registry = DefinitionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.total_count(), 0);
        assert!(registry.abilities.is_empty());
        assert!(registry.terrains.is_empty());
        assert!(registry.custom.is_empty());
    }

    #[test]
    fn test_definition_registry_bucket_access() {
        let mut registry = DefinitionRegistry::new();

        let id = DefinitionId::new("ter_000001");
        registry.terrains.insert(
            id.clone(),
            RegistryEntry::new(id.clone())
                .with_data(serde_json::json!({"name": "Grass", "move_cost": 1.0})),
        );

        assert_eq!(registry.total_count(), 1);
        assert!(!registry.terrains.is_empty());
        assert_eq!(registry.terrains.version(), 1);

        // Test dynamic bucket access
        let bucket = registry.bucket("terrains");
        assert!(bucket.is_some());
        assert_eq!(bucket.unwrap().len(), 1);

        let bucket_mut = registry.bucket_mut("terrains");
        assert!(bucket_mut.is_some());

        let nonexistent = registry.bucket("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_definition_id_conversions() {
        let id1 = DefinitionId::new("abl_000001");
        let id2: DefinitionId = "abl_000001".into();
        let id3: DefinitionId = String::from("abl_000001").into();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1.as_str(), "abl_000001");
        assert_eq!(id1.to_string(), "abl_000001");
    }

    #[test]
    fn test_registry_entry_lifecycle() {
        let mut entry = RegistryEntry::new("abl_000001");
        assert!(!entry.deprecated);
        assert!(entry.data.is_none());

        entry.mark_deprecated();
        assert!(entry.deprecated);

        entry.supersede("abl_000002");
        assert!(entry.superseded_by.is_some());
        assert_eq!(entry.superseded_by.as_ref().unwrap().as_str(), "abl_000002");
    }

    #[test]
    fn test_mark_changed() {
        let mut registry = DefinitionRegistry::new();
        assert!(registry.take_changed().is_none());

        registry.mark_changed("terrains");
        assert_eq!(registry.take_changed(), Some("terrains"));
        assert!(registry.take_changed().is_none());
    }

    #[test]
    fn test_bucket_clear() {
        let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
        bucket.insert("abl_000001", RegistryEntry::new("abl_000001"));
        bucket.insert("abl_000002", RegistryEntry::new("abl_000002"));
        assert_eq!(bucket.len(), 2);

        bucket.clear();
        assert!(bucket.is_empty());
    }

    #[test]
    fn test_definition_id_display() {
        let id = DefinitionId::new("eff_000042");
        assert_eq!(format!("{}", id), "eff_000042");
    }
}
