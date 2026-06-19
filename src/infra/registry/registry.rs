//! Registry 核心类型
//!
//! Definition 注册中心的核心数据结构：提供通用的类型化存储桶、
//! 变更追踪、查询索引以及完整的 DefinitionRegistry Resource。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md

use bevy::prelude::*;
use std::collections::HashMap;

// Re-export from shared::ids where DefinitionId now lives (ADR-046 compliance)
pub use crate::shared::ids::DefinitionId;

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
        self.items.get(&DefinitionId::new(id))
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
    pub(crate) abilities: RegistryBucket<RegistryEntry>,
    pub(crate) effects: RegistryBucket<RegistryEntry>,
    pub(crate) modifiers: RegistryBucket<RegistryEntry>,
    pub(crate) tags: RegistryBucket<RegistryEntry>,
    pub(crate) attributes: RegistryBucket<RegistryEntry>,
    pub(crate) triggers: RegistryBucket<RegistryEntry>,
    pub(crate) cues: RegistryBucket<RegistryEntry>,
    pub(crate) items: RegistryBucket<RegistryEntry>,
    pub(crate) spells: RegistryBucket<RegistryEntry>,
    pub(crate) buffs: RegistryBucket<RegistryEntry>,

    // ---- Domains (5) ----
    pub(crate) factions: RegistryBucket<RegistryEntry>,
    pub(crate) terrains: RegistryBucket<RegistryEntry>,
    pub(crate) recipes: RegistryBucket<RegistryEntry>,
    pub(crate) loot_tables: RegistryBucket<RegistryEntry>,
    pub(crate) quests: RegistryBucket<RegistryEntry>,

    /// 自定义扩展桶（Domain 可注册额外的 Def 类型）
    pub(crate) custom: RegistryBucket<RegistryEntry>,

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
///
/// TODO[P2][Content]: 待 Asset 层定型后注册热重载 Observer
///   当前事件定义已就绪，但无订阅者（各 Domain 尚未接入 Asset 管线）。
#[derive(Event, Debug, Clone, Reflect)]
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
