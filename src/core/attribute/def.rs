//! 属性定义注册表（ADR-031 §1）
//!
//! 基于新的 5+6 Linglan 属性模型：
//! - 核心五维：phys_atk, magic_atk, phys_def, magic_def, max_hp
//! - 次级六维：crit_rate, crit_dmg, move_range, atk_range, hit_rate, dodge_rate
//!
//! 所有数值边界从 RON 配置读取，禁止硬编码。
//! 所有数值使用 i32（百分比 = 万分比，如 50% = 5000）。

use crate::shared::ids::AttributeId;
use crate::shared::registry::loader::LoadError;
use crate::shared::registry::validatable::ValidationSeverity;
use crate::shared::registry::{
    LoadableSingleRegistry, Registry, RegistryInitStage, ValidatableRegistry, ValidationError,
};
use bevy::app::{App, Plugin, PreStartup};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// ============================================================================
// RON 反序列化类型
// ============================================================================

/// RON 文件顶层结构
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDefList {
    pub attributes: Vec<AttributeDef>,
}

/// RON 单条属性定义
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    pub id: String,
    pub name_key: String,
    pub category: AttributeCategory,
    pub default: i32,
    pub min: i32,
    pub max: i32,
}

// ============================================================================
// AttributeCategory
// ============================================================================

/// 属性分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum AttributeCategory {
    Core,
    Secondary,
}

// ============================================================================
// AttributeDefinition
// ============================================================================

/// 属性定义（运行时元数据，Definition 层，不可变）
#[derive(Debug, Clone)]
pub struct AttributeDefinition {
    /// 属性唯一标识
    pub id: AttributeId,
    /// 本地化 Key（用于 UI 显示）
    pub name_key: String,
    /// 属性分类
    pub category: AttributeCategory,
    /// 默认值
    pub default: i32,
    /// 最小值（含）
    pub min: i32,
    /// 最大值（含）
    pub max: i32,
}

impl AttributeDefinition {
    /// 将值钳制到 [min, max] 区间
    pub fn clamp(&self, value: i32) -> i32 {
        value.clamp(self.min, self.max)
    }
}

// ============================================================================
// AttributeRegistry
// ============================================================================

/// 属性注册表（Layer 1，零依赖）
///
/// 管理所有 AttributeDefinition，以 AttributeId 为 Key。
/// 加载完成后在运行时只读。
#[derive(Resource, Default, Debug)]
pub struct AttributeRegistry {
    definitions: HashMap<AttributeId, AttributeDefinition>,
}

impl Registry for AttributeRegistry {
    type Key = AttributeId;
    type Data = AttributeDefinition;

    fn len(&self) -> usize {
        self.definitions.len()
    }

    fn get(&self, key: &AttributeId) -> Option<&AttributeDefinition> {
        self.definitions.get(key)
    }

    fn keys(&self) -> Vec<&AttributeId> {
        self.definitions.keys().collect()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&AttributeId, &AttributeDefinition)> + '_> {
        Box::new(self.definitions.iter())
    }
}

impl LoadableSingleRegistry for AttributeRegistry {
    type Def = AttributeDefList;
    type Error = RegistryLoadError;

    fn register_def(&mut self, def: AttributeDefList) -> Result<(), Self::Error> {
        for attr_def in def.attributes {
            let id = AttributeId::new(&attr_def.id);
            if self.definitions.contains_key(&id) {
                return Err(RegistryLoadError::Duplicate(format!(
                    "Duplicate attribute definition: {}",
                    attr_def.id
                )));
            }
            self.definitions.insert(
                id,
                AttributeDefinition {
                    id,
                    name_key: attr_def.name_key,
                    category: attr_def.category,
                    default: attr_def.default,
                    min: attr_def.min,
                    max: attr_def.max,
                },
            );
        }
        Ok(())
    }
}

impl ValidatableRegistry for AttributeRegistry {
    fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (id, def) in &self.definitions {
            // min ≤ max
            if def.min > def.max {
                errors.push(ValidationError::error_for(
                    "AttributeRegistry",
                    id,
                    format!("min ({}) > max ({})", def.min, def.max),
                ));
            }
            // default ∈ [min, max]
            if def.default < def.min || def.default > def.max {
                errors.push(ValidationError::error_for(
                    "AttributeRegistry",
                    id,
                    format!(
                        "default ({}) outside range [{}, {}]",
                        def.default, def.min, def.max
                    ),
                ));
            }
            // max_hp must have min ≥ 1
            if id.as_str() == "max_hp" && def.min < 1 {
                errors.push(ValidationError::error_for(
                    "AttributeRegistry",
                    id,
                    "max_hp min must be ≥ 1",
                ));
            }
        }
        errors
    }
}

// ============================================================================
// RegistryLoadError
// ============================================================================

/// AttributeRegistry 加载错误
#[derive(Debug)]
pub enum RegistryLoadError {
    /// 重复的 ID
    Duplicate(String),
    /// 文件加载错误
    Load(LoadError),
}

impl std::fmt::Display for RegistryLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryLoadError::Duplicate(msg) => write!(f, "Duplicate attribute: {}", msg),
            RegistryLoadError::Load(e) => write!(f, "Load error: {}", e),
        }
    }
}

impl std::error::Error for RegistryLoadError {}

impl From<LoadError> for RegistryLoadError {
    fn from(e: LoadError) -> Self {
        RegistryLoadError::Load(e)
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// 属性注册表初始化 System
fn init_attribute_registry(mut commands: Commands) {
    let registry = match AttributeRegistry::load_from_file("content/attributes/attributes.ron") {
        Ok(reg) => {
            // 执行自校验
            let errors = reg.validate();
            if !errors.is_empty() {
                for err in &errors {
                    bevy::log::warn!(
                        target: "core",
                        "AttributeRegistry validation: {}",
                        err
                    );
                }
            }
            reg
        }
        Err(e) => {
            bevy::log::error!(
                target: "core",
                error = %e,
                "Failed to load AttributeRegistry, using defaults"
            );
            AttributeRegistry::default()
        }
    };
    commands.insert_resource(registry);
}

/// 属性注册表 Plugin
pub struct AttributeRegistryPlugin;

impl Plugin for AttributeRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            init_attribute_registry.in_set(RegistryInitStage::Layer1),
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
    fn registry_empty_by_default() {
        let reg = AttributeRegistry::default();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_load_from_file() {
        let reg = AttributeRegistry::load_from_file("content/attributes/attributes.ron").unwrap();
        assert_eq!(reg.len(), 11);

        let phys_atk = reg.get(&AttributeId::new("phys_atk")).unwrap();
        assert_eq!(phys_atk.category, AttributeCategory::Core);
        assert_eq!(phys_atk.default, 10);
        assert_eq!(phys_atk.min, 0);
        assert_eq!(phys_atk.max, 99999);

        let crit_rate = reg.get(&AttributeId::new("crit_rate")).unwrap();
        assert_eq!(crit_rate.category, AttributeCategory::Secondary);
        assert_eq!(crit_rate.default, 500);
        assert_eq!(crit_rate.min, 0);
        assert_eq!(crit_rate.max, 9500);
    }

    #[test]
    fn registry_contains_all_expected_ids() {
        let reg = AttributeRegistry::load_from_file("content/attributes/attributes.ron").unwrap();
        let expected_ids = [
            "phys_atk",
            "magic_atk",
            "phys_def",
            "magic_def",
            "max_hp",
            "crit_rate",
            "crit_dmg",
            "move_range",
            "atk_range",
            "hit_rate",
            "dodge_rate",
        ];
        for id_str in &expected_ids {
            assert!(
                reg.contains(&AttributeId::new(id_str)),
                "Missing attribute: {}",
                id_str
            );
        }
    }

    #[test]
    fn registry_get_nonexistent() {
        let reg = AttributeRegistry::default();
        assert!(reg.get(&AttributeId::new("nonexistent")).is_none());
    }

    #[test]
    fn registry_keys_and_iter() {
        let reg = AttributeRegistry::load_from_file("content/attributes/attributes.ron").unwrap();
        let keys = reg.keys();
        assert_eq!(keys.len(), 11);

        let entries: Vec<_> = reg.iter().collect();
        assert_eq!(entries.len(), 11);
    }

    #[test]
    fn validation_passes_for_valid_data() {
        let reg = AttributeRegistry::load_from_file("content/attributes/attributes.ron").unwrap();
        let errors = reg.validate();
        assert!(errors.is_empty(), "Validation errors: {:?}", errors);
    }

    #[test]
    fn validation_catches_min_greater_than_max() {
        let mut reg = AttributeRegistry::default();
        reg.definitions.insert(
            AttributeId::new("bad_attr"),
            AttributeDefinition {
                id: AttributeId::new("bad_attr"),
                name_key: "test".into(),
                category: AttributeCategory::Core,
                default: 50,
                min: 100,
                max: 10,
            },
        );
        let errors = reg.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("min")));
    }

    #[test]
    fn validation_catches_default_out_of_range() {
        let mut reg = AttributeRegistry::default();
        reg.definitions.insert(
            AttributeId::new("bad_default"),
            AttributeDefinition {
                id: AttributeId::new("bad_default"),
                name_key: "test".into(),
                category: AttributeCategory::Secondary,
                default: 99999,
                min: 0,
                max: 100,
            },
        );
        let errors = reg.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("default")));
    }

    #[test]
    fn clamp_value_to_boundaries() {
        let def = AttributeDefinition {
            id: AttributeId::new("test"),
            name_key: "test".into(),
            category: AttributeCategory::Core,
            default: 50,
            min: 0,
            max: 100,
        };
        assert_eq!(def.clamp(-10), 0);
        assert_eq!(def.clamp(50), 50);
        assert_eq!(def.clamp(200), 100);
    }
}
