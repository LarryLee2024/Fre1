//! Registry 基础类型与枚举
//!
//! 定义 Def 注册中心的 ID 类型、分配器、条目以及领域错误。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md §2。

use std::collections::HashMap;

/// Def ID 类型枚举。
///
/// 每种类型对应一个固定的 ID 前缀，用于 ID 分配和类型路由。
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
    /// 从 ID 前缀字符串解析 IdType，未知前缀返回 None。
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

    /// 返回 ID 类型对应的固定前缀（如 "abl_"、"eff_"）。
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

    /// 返回类型的人类可读名称，用于日志和错误信息。
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

/// ID 分配器状态——管理单个类型的编号递增。
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
    /// 创建分配器状态，从编号 1 开始分配。
    pub fn new(prefix: impl Into<String>, digit_count: u8) -> Self {
        Self {
            prefix: prefix.into(),
            next_id: 1,
            digit_count,
        }
    }

    /// 分配下一个 ID 并自动递增编号。
    ///
    /// 格式：`{prefix}{number:0>digit_count$}`，如 `abl_000001`。
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
///
/// 每个 IdType 对应一个 AllocatorState，按需注册后即可分配 ID。
#[derive(Debug, Clone, PartialEq)]
pub struct IdAllocator {
    /// 各类型的分配器状态
    pub allocators: HashMap<IdType, AllocatorState>,
}

impl IdAllocator {
    /// 创建空的 ID 分配器，需调用 register_type 后才能分配。
    pub fn new() -> Self {
        Self {
            allocators: HashMap::new(),
        }
    }

    /// 注册一个 ID 类型及其分配器状态。
    pub fn register_type(&mut self, id_type: IdType, state: AllocatorState) {
        self.allocators.insert(id_type, state);
    }

    /// 分配指定类型的下一个 ID，未注册的类型返回 None。
    pub fn allocate(&mut self, id_type: &IdType) -> Option<String> {
        self.allocators.get_mut(id_type).map(|s| s.allocate())
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// 注册条目元数据——存储 Def 的注册信息和废弃状态。
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
    /// 创建注册条目，def_id、def_type、data 必填。
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

    /// 标记为已废弃，废弃后运行时不再加载。
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    /// 设置替换者 ID，标记此 Def 被哪个新 Def 取代。
    pub fn superseded_by(mut self, new_id: impl Into<String>) -> Self {
        self.superseded_by = Some(new_id.into());
        self
    }
}
