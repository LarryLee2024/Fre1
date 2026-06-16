//! 标签定义注册表（ADR-031 §2）
//!
//! 基于 Linglan 5 分类标签模型：
//! - Elemental（元素/伤害类型）
//! - Status（状态/控制层级）
//! - Class（阵营/身份）
//! - Equipment（装备属性）
//! - Mechanism（底层机制）
//!
//! 标签定义从 RON 加载，互斥规则在 RON 中配置，禁止硬编码。

use crate::shared::ids::TagId;
use crate::shared::registry::loader::LoadError;
use crate::shared::registry::validatable::ValidationSeverity;
use crate::shared::registry::{
    LoadableSingleRegistry, Registry, RegistryInitStage, ValidatableRegistry, ValidationError,
};
use bevy::app::{App, Plugin, PreStartup};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use super::GameplayTag;

// ============================================================================
// TagCategory — 5 分类
// ============================================================================

/// 标签分类（领域验证折中 5 类方案）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum TagCategory {
    /// 元素/伤害类型：fire, ice, lightning, dmg_physical, dmg_magical, etc.
    Elemental,
    /// 状态/控制层级：buff, debuff, special_state, control_soft/hard/full
    Status,
    /// 阵营/身份：ally, enemy, summon, boss, mechanical
    Class,
    /// 装备属性：weapon_type, armor_type, equipment_attr
    Equipment,
    /// 底层机制：flying, grounded, dispellable, undispellable, reflectable
    Mechanism,
}

// ============================================================================
// RON 反序列化类型
// ============================================================================

/// RON 文件顶层结构
#[derive(Debug, Clone, Deserialize)]
pub struct TagDefList {
    pub tags: Vec<TagDef>,
    /// 互斥标签对
    #[serde(default)]
    pub mutual_exclusions: Vec<MutualExclusionDef>,
}

/// RON 单条标签定义
#[derive(Debug, Clone, Deserialize)]
pub struct TagDef {
    pub id: String,
    pub category: TagCategory,
    pub priority_weight: u32,
    pub dispellable: bool,
    pub reflectable: bool,
    /// 本地化 Key（可选）
    #[serde(default)]
    pub name_key: Option<String>,
}

/// RON 互斥规则定义
#[derive(Debug, Clone, Deserialize)]
pub struct MutualExclusionDef {
    pub tag_a: String,
    pub tag_b: String,
}

// ============================================================================
// TagDefinition
// ============================================================================

/// 标签定义（运行时元数据，Definition 层，不可变）
#[derive(Debug, Clone)]
pub struct TagDefinition {
    /// 标签唯一标识
    pub id: TagId,
    /// 对应的位掩码
    pub bitmask: GameplayTag,
    /// 本地化 Key
    pub name_key: Option<String>,
    /// 标签分类
    pub category: TagCategory,
    /// 优先级权重（控制类适用）
    pub priority_weight: u32,
    /// 是否可驱散
    pub dispellable: bool,
    /// 是否可反弹
    pub reflectable: bool,
    /// 互斥标签列表
    pub mutual_exclusions: Vec<TagId>,
}

// ============================================================================
// Bitmask 分配
// ============================================================================

/// 位掩码分配器：为每个 TagId 分配唯一的 bit 位置
///
/// 按分类分配不同的字节区间：
/// - Elemental:  bits 0-7
/// - Status:     bits 8-15
/// - Class:      bits 16-23
/// - Equipment:  bits 24-31
/// - Mechanism:  bits 32-39
fn allocate_bitmask(category: &TagCategory, index: usize) -> GameplayTag {
    let base = match category {
        TagCategory::Elemental => 0u64,
        TagCategory::Status => 8u64,
        TagCategory::Class => 16u64,
        TagCategory::Equipment => 24u64,
        TagCategory::Mechanism => 32u64,
    };
    GameplayTag::from_bits(1u64 << (base + index as u64))
}

// ============================================================================
// TagRegistry
// ============================================================================

/// 标签注册表（Layer 1，零依赖）
///
/// 管理所有 TagDefinition，支持 TagId ↔ GameplayTag 双向映射。
/// 从 RON 文件加载，运行时只读。
#[derive(Resource, Default, Debug)]
pub struct TagRegistry {
    /// TagId → TagDefinition
    definitions: HashMap<TagId, TagDefinition>,
    /// GameplayTag → TagId（反向映射）
    bitmask_to_id: HashMap<GameplayTag, TagId>,
    /// 互斥规则列表
    mutual_exclusions: Vec<(TagId, TagId)>,
}

impl Registry for TagRegistry {
    type Key = TagId;
    type Data = TagDefinition;

    fn len(&self) -> usize {
        self.definitions.len()
    }

    fn get(&self, key: &TagId) -> Option<&TagDefinition> {
        self.definitions.get(key)
    }

    fn keys(&self) -> Vec<&TagId> {
        self.definitions.keys().collect()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&TagId, &TagDefinition)> + '_> {
        Box::new(self.definitions.iter())
    }
}

impl TagRegistry {
    /// 通过 GameplayTag 查询 TagId
    pub fn id_for_bitmask(&self, bitmask: GameplayTag) -> Option<&TagId> {
        self.bitmask_to_id.get(&bitmask)
    }

    /// 通过 TagId 查询 GameplayTag 位掩码
    pub fn bitmask_for_id(&self, id: &TagId) -> Option<GameplayTag> {
        self.definitions.get(id).map(|def| def.bitmask)
    }

    /// 检查两个标签是否互斥
    pub fn are_mutually_exclusive(&self, a: &TagId, b: &TagId) -> bool {
        self.mutual_exclusions
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
    }

    /// 按分类查询标签
    pub fn tags_by_category(&self, category: TagCategory) -> Vec<&TagId> {
        self.definitions
            .iter()
            .filter(|(_, def)| def.category == category)
            .map(|(id, _)| id)
            .collect()
    }

    /// 返回所有互斥规则
    pub fn mutual_exclusion_rules(&self) -> &[(TagId, TagId)] {
        &self.mutual_exclusions
    }

    /// 检查位掩码使用率（≥80% 时返回 true）
    pub fn bit_mask_usage_warning(&self) -> bool {
        let used_bits = self.definitions.len();
        used_bits >= 38 // 48 * 0.8 = 38.4 (we use 48 bits of the u64)
    }
}

// ============================================================================
// LoadableSingleRegistry impl
// ============================================================================

/// TagRegistry 加载错误
#[derive(Debug)]
pub enum RegistryLoadError {
    Duplicate(String),
    Load(LoadError),
    UnknownTag(String),
}

impl std::fmt::Display for RegistryLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryLoadError::Duplicate(msg) => write!(f, "Duplicate tag: {}", msg),
            RegistryLoadError::Load(e) => write!(f, "Load error: {}", e),
            RegistryLoadError::UnknownTag(msg) => write!(f, "Unknown tag: {}", msg),
        }
    }
}

impl std::error::Error for RegistryLoadError {}

impl From<LoadError> for RegistryLoadError {
    fn from(e: LoadError) -> Self {
        RegistryLoadError::Load(e)
    }
}

impl LoadableSingleRegistry for TagRegistry {
    type Def = TagDefList;
    type Error = RegistryLoadError;

    fn register_def(&mut self, def: TagDefList) -> Result<(), Self::Error> {
        // 统计每个分类的标签数，用于位掩码分配
        let mut category_counts: HashMap<TagCategory, usize> = HashMap::new();

        // Phase 1: 注册所有标签
        for tag_def in &def.tags {
            let id = TagId::new(&tag_def.id);
            if self.definitions.contains_key(&id) {
                return Err(RegistryLoadError::Duplicate(tag_def.id.clone()));
            }
            let count = category_counts.entry(tag_def.category).or_insert(0);
            let bitmask = allocate_bitmask(&tag_def.category, *count);
            *count += 1;

            let definition = TagDefinition {
                id: id.clone(),
                bitmask,
                name_key: tag_def.name_key.clone(),
                category: tag_def.category,
                priority_weight: tag_def.priority_weight,
                dispellable: tag_def.dispellable,
                reflectable: tag_def.reflectable,
                mutual_exclusions: Vec::new(), // 第二遍填
            };

            self.bitmask_to_id.insert(bitmask, id.clone());
            self.definitions.insert(id, definition);
        }

        // Phase 2: 注册互斥规则
        for rule in &def.mutual_exclusions {
            let id_a = TagId::new(&rule.tag_a);
            let id_b = TagId::new(&rule.tag_b);

            if !self.definitions.contains_key(&id_a) {
                return Err(RegistryLoadError::UnknownTag(rule.tag_a.clone()));
            }
            if !self.definitions.contains_key(&id_b) {
                return Err(RegistryLoadError::UnknownTag(rule.tag_b.clone()));
            }

            // 将互斥信息写入双方
            if let Some(def_a) = self.definitions.get_mut(&id_a) {
                def_a.mutual_exclusions.push(id_b.clone());
            }
            if let Some(def_b) = self.definitions.get_mut(&id_b) {
                def_b.mutual_exclusions.push(id_a.clone());
            }

            self.mutual_exclusions.push((id_a, id_b));
        }

        Ok(())
    }
}

impl ValidatableRegistry for TagRegistry {
    fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // 检查互斥规则一致性
        for (a, b) in &self.mutual_exclusions {
            if !self.definitions.contains_key(a) {
                errors.push(ValidationError::error_for(
                    "TagRegistry",
                    a,
                    "Referenced in mutual exclusion but not registered",
                ));
            }
            if !self.definitions.contains_key(b) {
                errors.push(ValidationError::error_for(
                    "TagRegistry",
                    b,
                    "Referenced in mutual exclusion but not registered",
                ));
            }
        }

        // 检查位掩码使用率
        if self.bit_mask_usage_warning() {
            errors.push(ValidationError::warning(
                "TagRegistry",
                format!(
                    "Bitmask usage at {} bits, consider migrating to hierarchical tags",
                    self.definitions.len()
                ),
            ));
        }

        errors
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// 标签注册表初始化 System
fn init_tag_registry(mut commands: Commands) {
    let registry = match TagRegistry::load_from_file("content/tags/tags.ron") {
        Ok(reg) => {
            let errors = reg.validate();
            if !errors.is_empty() {
                for err in &errors {
                    bevy::log::warn!(target: "core", "TagRegistry validation: {}", err);
                }
            }
            reg
        }
        Err(e) => {
            bevy::log::error!(
                target: "core",
                error = %e,
                "Failed to load TagRegistry, using defaults"
            );
            TagRegistry::default()
        }
    };
    commands.insert_resource(registry);
}

/// 标签注册表 Plugin
pub struct TagRegistryPlugin;

impl Plugin for TagRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            init_tag_registry.in_set(RegistryInitStage::Layer1),
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::registry::Registry;

    #[test]
    fn tag_registry_load_from_file() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();
        assert!(!reg.is_empty());

        // Verify a few known tags
        let dmg_fire = TagId::new("dmg_fire");
        let def = reg.get(&dmg_fire).unwrap();
        assert_eq!(def.category, TagCategory::Elemental);
        assert!(!def.dispellable);
        assert!(def.reflectable);

        let control_full = TagId::new("control_full");
        let def = reg.get(&control_full).unwrap();
        assert_eq!(def.category, TagCategory::Status);
        assert_eq!(def.priority_weight, 3);
    }

    #[test]
    fn tag_registry_bitmask_mapping() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();

        // Check bidirectionality
        let id = TagId::new("dmg_fire");
        let bitmask = reg.bitmask_for_id(&id).unwrap();
        let resolved = reg.id_for_bitmask(bitmask).unwrap();
        assert_eq!(*resolved, id);
    }

    #[test]
    fn tag_registry_mutual_exclusions() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();

        let flying = TagId::new("flying");
        let grounded = TagId::new("grounded");
        assert!(reg.are_mutually_exclusive(&flying, &grounded));
        assert!(reg.are_mutually_exclusive(&grounded, &flying));

        // Non-exclusive pair
        let dmg_fire = TagId::new("dmg_fire");
        assert!(!reg.are_mutually_exclusive(&flying, &dmg_fire));
    }

    #[test]
    fn tag_registry_tags_by_category() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();
        let elemental = reg.tags_by_category(TagCategory::Elemental);
        assert_eq!(elemental.len(), 6); // 6 damage type tags

        let status = reg.tags_by_category(TagCategory::Status);
        assert_eq!(status.len(), 8); // 8 status tags
    }

    #[test]
    fn tag_registry_duplicate_rejected() {
        let result = TagRegistry::load_from_file("content/tags/tags.ron");
        assert!(result.is_ok()); // single load should work

        // Now manually test duplicate detection
        let mut reg = TagRegistry::default();
        let list = TagDefList {
            tags: vec![
                TagDef {
                    id: "dup".into(),
                    category: TagCategory::Elemental,
                    priority_weight: 0,
                    dispellable: false,
                    reflectable: false,
                    name_key: None,
                },
                TagDef {
                    id: "dup".into(),
                    category: TagCategory::Status,
                    priority_weight: 0,
                    dispellable: false,
                    reflectable: false,
                    name_key: None,
                },
            ],
            mutual_exclusions: vec![],
        };
        let result = reg.register_def(list);
        assert!(result.is_err());
    }

    #[test]
    fn tag_registry_unknown_tag_in_exclusion() {
        let mut reg = TagRegistry::default();
        let list = TagDefList {
            tags: vec![TagDef {
                id: "known".into(),
                category: TagCategory::Elemental,
                priority_weight: 0,
                dispellable: false,
                reflectable: false,
                name_key: None,
            }],
            mutual_exclusions: vec![MutualExclusionDef {
                tag_a: "known".into(),
                tag_b: "unknown".into(),
            }],
        };
        let result = reg.register_def(list);
        assert!(result.is_err());
    }

    #[test]
    fn tag_registry_distinct_bitmasks() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();

        // All bitmasks should be unique
        let mut seen = std::collections::HashSet::new();
        for (id, def) in reg.iter() {
            assert!(
                seen.insert(def.bitmask),
                "Duplicate bitmask for tag {}: {:?}",
                id,
                def.bitmask
            );
        }
    }

    #[test]
    fn tag_registry_validation_passes() {
        let reg = TagRegistry::load_from_file("content/tags/tags.ron").unwrap();
        let errors = reg.validate();
        // Should only have warnings at most
        assert!(
            errors
                .iter()
                .all(|e| e.severity != ValidationSeverity::Error),
            "Unexpected error validation errors: {:?}",
            errors
        );
    }
}
