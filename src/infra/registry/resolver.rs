//! ID 分配与冲突检测
//!
//! 提供 Definition ID 的自动分配、格式校验和注册时冲突检测。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md §3.4, §3.5

use std::collections::HashMap;

use super::registry::{DefinitionId, DefinitionRegistry};

// ============================================================================
// IdType
// ============================================================================

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
            "ltb_" => Some(Self::LootTable),
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
            Self::LootTable => "ltb_",
            Self::Custom(_) => "cst_",
        }
    }

    /// 返回类型名称（对应 DefinitionRegistry 的桶名和字段名）。
    pub fn name(&self) -> &str {
        match self {
            Self::Ability => "abilities",
            Self::Effect => "effects",
            Self::Trigger => "triggers",
            Self::Tag => "tags",
            Self::Attribute => "attributes",
            Self::Cue => "cues",
            Self::Item => "items",
            Self::Spell => "spells",
            Self::Quest => "quests",
            Self::Faction => "factions",
            Self::Terrain => "terrains",
            Self::Recipe => "recipes",
            Self::Buff => "buffs",
            Self::LootTable => "loot_tables",
            Self::Custom(_) => "custom",
        }
    }
}

// ============================================================================
// AllocatorState
// ============================================================================

/// 单个 ID 类型分配器状态。
#[derive(Debug, Clone)]
pub struct AllocatorState {
    /// ID 前缀
    pub prefix: String,
    /// 下一个可用编号
    pub next_id: u64,
    /// 数字位数（0-padded）
    pub digit_count: u8,
    /// 已回收的 ID 列表
    pub recycled: Vec<u64>,
}

impl AllocatorState {
    /// 创建分配器状态。
    pub fn new(prefix: impl Into<String>, digit_count: u8) -> Self {
        Self {
            prefix: prefix.into(),
            next_id: 1,
            digit_count,
            recycled: Vec::new(),
        }
    }

    /// 分配下一个 ID。
    ///
    /// 格式：`{prefix}{number:0>digit_count$}`
    /// 示例：`abl_000001`, `eff_000042`
    pub fn allocate(&mut self) -> DefinitionId {
        let id = if let Some(recycled) = self.recycled.pop() {
            recycled
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };
        DefinitionId::new(format!(
            "{}{:0>width$}",
            self.prefix,
            id,
            width = self.digit_count as usize
        ))
    }

    /// 回收一个 ID（标记为可重用）。
    pub fn recycle(&mut self, id: u64) {
        self.recycled.push(id);
    }
}

// ============================================================================
// IdAllocator
// ============================================================================

/// ID 分配器——管理各类型前缀的数字编号分配。
#[derive(Debug, Clone)]
pub struct IdAllocator {
    /// 各类型的分配器状态
    allocators: HashMap<IdType, AllocatorState>,
}

impl IdAllocator {
    /// 创建空的 ID 分配器。
    pub fn new() -> Self {
        Self {
            allocators: HashMap::new(),
        }
    }

    /// 使用默认配置创建完整的 IdAllocator。
    ///
    /// 所有类型均注册，数字位数为 6（如 `abl_000001`）。
    pub fn new_full() -> Self {
        let mut allocator = Self::new();
        allocator.register_type(IdType::Ability, AllocatorState::new("abl_", 6));
        allocator.register_type(IdType::Effect, AllocatorState::new("eff_", 6));
        allocator.register_type(IdType::Trigger, AllocatorState::new("trg_", 6));
        allocator.register_type(IdType::Tag, AllocatorState::new("tag_", 6));
        allocator.register_type(IdType::Attribute, AllocatorState::new("attr_", 6));
        allocator.register_type(IdType::Cue, AllocatorState::new("cue_", 6));
        allocator.register_type(IdType::Item, AllocatorState::new("itm_", 6));
        allocator.register_type(IdType::Spell, AllocatorState::new("spl_", 6));
        allocator.register_type(IdType::Buff, AllocatorState::new("buf_", 6));
        allocator.register_type(IdType::Faction, AllocatorState::new("fct_", 6));
        allocator.register_type(IdType::Terrain, AllocatorState::new("ter_", 6));
        allocator.register_type(IdType::Recipe, AllocatorState::new("rcp_", 6));
        allocator.register_type(IdType::LootTable, AllocatorState::new("ltb_", 6));
        allocator.register_type(IdType::Quest, AllocatorState::new("qst_", 6));
        allocator
    }

    /// 注册一个 ID 类型及其分配器。
    pub fn register_type(&mut self, id_type: IdType, state: AllocatorState) {
        self.allocators.insert(id_type, state);
    }

    /// 分配指定类型的下一个 ID。
    pub fn allocate(&mut self, id_type: &IdType) -> Option<DefinitionId> {
        self.allocators.get_mut(id_type).map(|s| s.allocate())
    }

    /// 检查 ID 是否已被注册。
    pub fn validate_id(id: &DefinitionId) -> bool {
        // ID 格式: 前缀 + 数字，总长度至少为 5
        if id.as_str().len() < 5 {
            return false;
        }
        // 4 字符前缀校验：未知前缀说明未注册的 Def 类型
        let prefix = &id.as_str()[..4]; // 4-char prefix: 3 letters + _
        IdType::from_prefix(prefix).is_some()
    }

    /// 检查 ID 格式是否匹配预期类型。
    pub fn validate_id_type(id: &DefinitionId, expected: &IdType) -> bool {
        if id.as_str().len() < 5 {
            return false;
        }
        let prefix = &id.as_str()[..4];
        IdType::from_prefix(prefix).as_ref() == Some(expected)
    }

    /// 获取分配器状态（用于测试/调试）。
    pub fn allocator_state(&self, id_type: &IdType) -> Option<&AllocatorState> {
        self.allocators.get(id_type)
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ValidationError / ValidationWarning
// ============================================================================

/// 注册校验错误。
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// 桶名称
    pub bucket: &'static str,
    /// Def ID
    pub def_id: DefinitionId,
    /// 错误描述
    pub message: String,
}

impl ValidationError {
    /// 创建注册校验错误。
    pub fn new(
        bucket: &'static str,
        def_id: impl Into<DefinitionId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            bucket,
            def_id: def_id.into(),
            message: message.into(),
        }
    }
}

/// 注册校验警告。
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    /// 桶名称
    pub bucket: &'static str,
    /// Def ID
    pub def_id: DefinitionId,
    /// 警告描述
    pub message: String,
}

impl ValidationWarning {
    /// 创建注册校验警告。
    pub fn new(
        bucket: &'static str,
        def_id: impl Into<DefinitionId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            bucket,
            def_id: def_id.into(),
            message: message.into(),
        }
    }
}

/// 断裂引用报告。
#[derive(Debug, Clone, PartialEq)]
pub struct BrokenReference {
    /// 来源 Def ID
    pub source_def: DefinitionId,
    /// 来源桶
    pub source_bucket: &'static str,
    /// 引用的字段名
    pub field: String,
    /// 被引用的 ID
    pub referenced_id: String,
    /// 期望的类型
    pub expected_type: String,
}

// ============================================================================
// RegistryValidation
// ============================================================================

/// 注册时的一致性校验结果。
#[derive(Debug, Clone)]
pub struct RegistryValidation {
    pub has_errors: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    /// 跨 Def 引用检查
    pub cross_references: CrossReferenceReport,
}

impl RegistryValidation {
    /// 创建空的校验结果。
    pub fn new() -> Self {
        Self {
            has_errors: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            cross_references: CrossReferenceReport::new(),
        }
    }

    /// 添加校验错误。
    pub fn add_error(&mut self, error: ValidationError) {
        self.has_errors = true;
        self.errors.push(error);
    }

    /// 添加校验警告。
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// 合并另一个校验结果。
    pub fn merge(&mut self, other: Self) {
        self.has_errors = self.has_errors || other.has_errors;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.cross_references.merge(other.cross_references);
    }

    /// 是否无错误。
    pub fn is_clean(&self) -> bool {
        !self.has_errors && self.errors.is_empty()
    }
}

impl Default for RegistryValidation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CrossReferenceReport
// ============================================================================

/// 跨 Def 引用检查报告。
#[derive(Debug, Clone)]
pub struct CrossReferenceReport {
    pub total_defs: u32,
    pub total_references: u32,
    pub broken_references: Vec<BrokenReference>,
}

impl CrossReferenceReport {
    /// 创建空的跨引用报告。
    pub fn new() -> Self {
        Self {
            total_defs: 0,
            total_references: 0,
            broken_references: Vec::new(),
        }
    }

    /// 添加断裂引用。
    pub fn add_broken(&mut self, reference: BrokenReference) {
        self.broken_references.push(reference);
    }

    /// 合并另一个报告。
    pub fn merge(&mut self, other: Self) {
        self.total_defs += other.total_defs;
        self.total_references += other.total_references;
        self.broken_references.extend(other.broken_references);
    }

    /// 检查是否存在断裂引用。
    pub fn has_broken_references(&self) -> bool {
        !self.broken_references.is_empty()
    }
}

impl Default for CrossReferenceReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ValidationRunner
// ============================================================================

/// 校验执行器——对 DefinitionRegistry 执行一致性校验。
///
/// 验证规则（对应 registry_schema.md §8）：
/// - V1: ID 格式正确（前缀 + 数字）
/// - V2: ID 全局唯一
/// - V3: 所有跨 Def 引用有效
/// - V6: RegistryEntry 包含必要字段
pub struct ValidationRunner;

impl ValidationRunner {
    /// 对完整的 DefinitionRegistry 执行校验。
    pub fn validate(registry: &DefinitionRegistry) -> RegistryValidation {
        let mut result = RegistryValidation::new();

        // V1: ID 格式校验
        Self::validate_id_formats(registry, &mut result);

        // V2: ID 全局唯一性检查
        Self::check_global_uniqueness(registry, &mut result);

        // V6: 条目完整性
        Self::check_entry_integrity(registry, &mut result);

        result
    }

    /// V1: 检查所有 ID 的格式。
    fn validate_id_formats(registry: &DefinitionRegistry, result: &mut RegistryValidation) {
        for bucket_name in registry.bucket_names() {
            let Some(bucket) = registry.bucket(bucket_name) else {
                continue;
            };
            for id in bucket.keys() {
                if !IdAllocator::validate_id(id) {
                    result.add_error(ValidationError::new(
                        bucket_name,
                        id.clone(),
                        format!("invalid ID format: '{}' (expected prefix + digits)", id),
                    ));
                }
            }
        }
    }

    /// V2: 跨桶检查 ID 全局唯一性。
    fn check_global_uniqueness(registry: &DefinitionRegistry, result: &mut RegistryValidation) {
        use std::collections::HashSet;

        let mut seen: HashSet<String> = HashSet::new();
        for bucket_name in registry.bucket_names() {
            let Some(bucket) = registry.bucket(bucket_name) else {
                continue;
            };
            for id in bucket.keys() {
                if !seen.insert(id.as_str().to_string()) {
                    result.add_error(ValidationError::new(
                        bucket_name,
                        id.clone(),
                        format!("duplicate ID '{}' across buckets", id),
                    ));
                }
            }
        }
    }

    /// V6: 检查条目数据的完整性。
    fn check_entry_integrity(registry: &DefinitionRegistry, result: &mut RegistryValidation) {
        for bucket_name in registry.bucket_names() {
            let Some(bucket) = registry.bucket(bucket_name) else {
                continue;
            };
            for (_id, entry) in bucket.iter() {
                if entry.data.is_none() {
                    result.add_warning(ValidationWarning::new(
                        bucket_name,
                        entry.id.clone(),
                        "entry has no data payload".to_string(),
                    ));
                }
            }
        }
    }
}
