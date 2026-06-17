//! Registry 基础类型与枚举
//!
//! 定义 Def 注册中心的 ID 类型、分配器、条目以及领域错误。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md §2。

use std::collections::HashMap;

/// Def ID 类型枚举。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdType {
    Ability,
    Effect,
    Trigger,
    Tag,
    Attribute,
    Cue,
    Item,
    Spell,
    Quest,
    Faction,
    Terrain,
    Recipe,
    Buff,
    LootTable,
    Custom(String),
}

impl IdType {
    /// 从类型前缀字符串解析 IdType。
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "abl_" => Some(Self::Ability),
            "eff_" => Some(Self::Effect),
            "trg_" => Some(Self::Trigger),
            "tag_" => Some(Self::Tag),
            "attr_" => Some(Self::Attribute),
            "cue_" => Some(Self::Cue),
            "itm_" => Some(Self::Item),
            "spl_" => Some(Self::Spell),
            "qst_" => Some(Self::Quest),
            "fct_" => Some(Self::Faction),
            "ter_" => Some(Self::Terrain),
            "rcp_" => Some(Self::Recipe),
            "buf_" => Some(Self::Buff),
            "oot_" => Some(Self::LootTable),
            _ => None,
        }
    }

    /// 返回 ID 类型的前缀。
    pub fn prefix(&self) -> &str {
        match self {
            Self::Ability => "abl_",
            Self::Effect => "eff_",
            Self::Trigger => "trg_",
            Self::Tag => "tag_",
            Self::Attribute => "attr_",
            Self::Cue => "cue_",
            Self::Item => "itm_",
            Self::Spell => "spl_",
            Self::Quest => "qst_",
            Self::Faction => "fct_",
            Self::Terrain => "ter_",
            Self::Recipe => "rcp_",
            Self::Buff => "buf_",
            Self::LootTable => "oot_",
            Self::Custom(_) => "cst_",
        }
    }

    /// 返回类型名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Ability => "Ability",
            Self::Effect => "Effect",
            Self::Trigger => "Trigger",
            Self::Tag => "Tag",
            Self::Attribute => "Attribute",
            Self::Cue => "Cue",
            Self::Item => "Item",
            Self::Spell => "Spell",
            Self::Quest => "Quest",
            Self::Faction => "Faction",
            Self::Terrain => "Terrain",
            Self::Recipe => "Recipe",
            Self::Buff => "Buff",
            Self::LootTable => "LootTable",
            Self::Custom(name) => name.as_str(),
        }
    }
}

/// ID 分配器状态。
#[derive(Debug, Clone, PartialEq)]
pub struct AllocatorState {
    /// 类型前缀
    pub prefix: String,
    /// 当前最大已分配编号
    pub next_id: u64,
    /// 数字位数（0-padded）
    pub digit_count: u8,
}

impl AllocatorState {
    /// 创建分配器状态。
    pub fn new(prefix: impl Into<String>, digit_count: u8) -> Self {
        Self {
            prefix: prefix.into(),
            next_id: 1,
            digit_count,
        }
    }

    /// 分配下一个 ID。
    ///
    /// 格式：`{prefix}{number:0>digit_count$}`
    /// 示例：`abl_000001`, `eff_000042`
    pub fn allocate(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!(
            "{}{:0>width$}",
            self.prefix,
            id,
            width = self.digit_count as usize
        )
    }
}

/// ID 分配器——管理各类型前缀的编号分配。
#[derive(Debug, Clone, PartialEq)]
pub struct IdAllocator {
    /// 各类型的分配器状态
    pub allocators: HashMap<IdType, AllocatorState>,
}

impl IdAllocator {
    /// 创建空的 ID 分配器。
    pub fn new() -> Self {
        Self {
            allocators: HashMap::new(),
        }
    }

    /// 注册一个 ID 类型及其分配器。
    pub fn register_type(&mut self, id_type: IdType, state: AllocatorState) {
        self.allocators.insert(id_type, state);
    }

    /// 分配指定类型的下一个 ID。
    pub fn allocate(&mut self, id_type: &IdType) -> Option<String> {
        self.allocators.get_mut(id_type).map(|s| s.allocate())
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// 注册条目元数据。
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryEntry {
    /// Def ID
    pub def_id: String,
    /// Def 类型
    pub def_type: String,
    /// Def 数据（序列化字符串或 JSON）
    pub data: String,
    /// 是否为 Deprecated
    pub deprecated: bool,
    /// 替换者 ID（如果被取代）
    pub superseded_by: Option<String>,
}

impl RegistryEntry {
    /// 创建注册条目。
    pub fn new(
        def_id: impl Into<String>,
        def_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            def_id: def_id.into(),
            def_type: def_type.into(),
            data: data.into(),
            deprecated: false,
            superseded_by: None,
        }
    }

    /// 标记为已废弃。
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    /// 设置替换者。
    pub fn superseded_by(mut self, new_id: impl Into<String>) -> Self {
        self.superseded_by = Some(new_id.into());
        self
    }
}

/// Registry 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    /// ID 已存在
    DuplicateId(String),
    /// ID 不存在
    IdNotFound(String),
    /// ID 格式无效
    InvalidIdFormat(String),
    /// 跨 Def 引用断裂
    BrokenReference {
        source: String,
        field: String,
        target: String,
    },
    /// 分配器未注册
    AllocatorNotFound(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate registry ID: {}", id),
            Self::IdNotFound(id) => write!(f, "registry ID not found: {}", id),
            Self::InvalidIdFormat(id) => write!(f, "invalid ID format: {}", id),
            Self::BrokenReference {
                source,
                field,
                target,
            } => {
                write!(
                    f,
                    "broken reference: {}.{} → {} (not found)",
                    source, field, target
                )
            }
            Self::AllocatorNotFound(msg) => write!(f, "allocator not found: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_id_type_prefix() {
        assert_eq!(IdType::Ability.prefix(), "abl_");
        assert_eq!(IdType::Effect.prefix(), "eff_");
        assert_eq!(IdType::Cue.prefix(), "cue_");
    }

    #[test]
    fn unit_002_id_type_from_prefix() {
        assert_eq!(IdType::from_prefix("abl_"), Some(IdType::Ability));
        assert_eq!(IdType::from_prefix("eff_"), Some(IdType::Effect));
        assert_eq!(IdType::from_prefix("xxx_"), None);
    }

    #[test]
    fn unit_003_id_type_name() {
        assert_eq!(IdType::Ability.name(), "Ability");
        assert_eq!(IdType::Custom("Test".into()).name(), "Test");
    }

    #[test]
    fn unit_004_allocator_allocate() {
        let mut state = AllocatorState::new("abl_", 6);
        assert_eq!(state.allocate(), "abl_000001");
        assert_eq!(state.allocate(), "abl_000002");
        assert_eq!(state.next_id, 3);
    }

    #[test]
    fn unit_005_allocator_different_digits() {
        let mut state = AllocatorState::new("eff_", 4);
        assert_eq!(state.allocate(), "eff_0001");
    }

    #[test]
    fn unit_006_id_allocator() {
        let mut alloc = IdAllocator::new();
        alloc.register_type(IdType::Ability, AllocatorState::new("abl_", 6));
        alloc.register_type(IdType::Effect, AllocatorState::new("eff_", 6));

        assert_eq!(alloc.allocate(&IdType::Ability), Some("abl_000001".into()));
        assert_eq!(alloc.allocate(&IdType::Effect), Some("eff_000001".into()));
        assert_eq!(alloc.allocate(&IdType::Ability), Some("abl_000002".into()));
    }

    #[test]
    fn unit_007_id_allocator_unregistered() {
        let mut alloc = IdAllocator::new();
        assert_eq!(alloc.allocate(&IdType::Ability), None);
    }

    #[test]
    fn unit_008_registry_entry() {
        let entry = RegistryEntry::new("abl_000001", "Ability", "name=Fireball,damage=50");
        assert_eq!(entry.def_id, "abl_000001");
        assert_eq!(entry.def_type, "Ability");
        assert!(!entry.deprecated);
    }

    #[test]
    fn unit_009_registry_entry_deprecated() {
        let entry = RegistryEntry::new("abl_000001", "Ability", "")
            .deprecated()
            .superseded_by("abl_000042");
        assert!(entry.deprecated);
        assert_eq!(entry.superseded_by, Some("abl_000042".into()));
    }

    #[test]
    fn unit_010_error_display() {
        let err = RegistryError::DuplicateId("abl_000001".into());
        let msg = format!("{}", err);
        assert!(msg.contains("abl_000001"));
    }

    #[test]
    fn unit_011_error_broken_reference() {
        let err = RegistryError::BrokenReference {
            source: "abl_000001".into(),
            field: "effect_id".into(),
            target: "eff_999999".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("eff_999999"));
    }
}
