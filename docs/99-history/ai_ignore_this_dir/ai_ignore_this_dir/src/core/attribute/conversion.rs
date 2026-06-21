//! 属性转换机制（ADR-031 §3）
//!
//! 支持 Linglan 中间层属性转换，在基础属性之后、最终属性之前结算。
//!
//! 典型转换规则：
//! - 防御转攻击：phys_def → phys_atk，比例 50%
//! - 损失血量转攻击：lost_hp → phys_atk，比例 30%
//!
//! 转换后的属性值可被后续百分比加成放大。
//! 条件型转换不参与基础属性面板计算。

use crate::shared::ids::AttributeId;
use crate::shared::registry::loader::LoadError;
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
pub struct ConversionDefList {
    pub conversions: Vec<ConversionDef>,
}

/// RON 单条转换定义
#[derive(Debug, Clone, Deserialize)]
pub struct ConversionDef {
    pub source: String,
    pub target: String,
    /// 转换比例（万分比，如 5000 = 50%）
    pub ratio: i32,
    /// 触发条件（可选）
    pub condition: Option<String>,
}

// ============================================================================
// AttributeConversion
// ============================================================================

/// 属性转换规则（运行时，不可变）
#[derive(Debug, Clone)]
pub struct AttributeConversion {
    /// 源属性 ID
    pub source: AttributeId,
    /// 目标属性 ID
    pub target: AttributeId,
    /// 转换比例（万分比）
    pub ratio: i32,
    /// 触发条件（可选，条件查询字符串如 "hp_below_50"）
    pub condition: Option<String>,
}

impl AttributeConversion {
    /// 执行转换计算：source_value * ratio / 10000
    pub fn calculate(&self, source_value: i32) -> i32 {
        (source_value as i64 * self.ratio as i64 / 10000) as i32
    }
}

// ============================================================================
// ConversionRegistry
// ============================================================================

/// 属性转换注册表（Layer 1）
///
/// 存储所有属性转换规则，按 source 属性分组。
#[derive(Resource, Default, Debug)]
pub struct ConversionRegistry {
    /// key = source AttributeId, value = Vec of conversion rules
    conversions: HashMap<AttributeId, Vec<AttributeConversion>>,
}

impl Registry for ConversionRegistry {
    type Key = AttributeId;
    type Data = Vec<AttributeConversion>;

    fn len(&self) -> usize {
        self.conversions.len()
    }

    fn get(&self, key: &AttributeId) -> Option<&Vec<AttributeConversion>> {
        self.conversions.get(key)
    }

    fn keys(&self) -> Vec<&AttributeId> {
        self.conversions.keys().collect()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&AttributeId, &Vec<AttributeConversion>)> + '_> {
        Box::new(self.conversions.iter())
    }
}

impl LoadableSingleRegistry for ConversionRegistry {
    type Def = ConversionDefList;
    type Error = ConversionLoadError;

    fn register_def(&mut self, def: ConversionDefList) -> Result<(), Self::Error> {
        for conv_def in def.conversions {
            let source = AttributeId::new(&conv_def.source);
            let target = AttributeId::new(&conv_def.target);

            let condition = conv_def.condition;

            let conversion = AttributeConversion {
                source: source.clone(),
                target,
                ratio: conv_def.ratio,
                condition,
            };

            self.conversions.entry(source).or_default().push(conversion);
        }
        Ok(())
    }
}

impl ValidatableRegistry for ConversionRegistry {
    fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (source, rules) in &self.conversions {
            for (i, rule) in rules.iter().enumerate() {
                // ratio must be positive
                if rule.ratio <= 0 {
                    errors.push(ValidationError::error_for(
                        "ConversionRegistry",
                        format!("{}[{}]", source, i),
                        format!("ratio must be positive, got {}", rule.ratio),
                    ));
                }
                // source and target shouldn't be the same
                if rule.source == rule.target {
                    errors.push(ValidationError::error_for(
                        "ConversionRegistry",
                        format!("{}[{}]", source, i),
                        "source and target are the same attribute",
                    ));
                }
            }
        }
        errors
    }
}

// ============================================================================
// ConversionLoadError
// ============================================================================

#[derive(Debug)]
pub enum ConversionLoadError {
    Load(LoadError),
    Duplicate(String),
}

impl std::fmt::Display for ConversionLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionLoadError::Load(e) => write!(f, "Load error: {}", e),
            ConversionLoadError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
        }
    }
}

impl std::error::Error for ConversionLoadError {}

impl From<LoadError> for ConversionLoadError {
    fn from(e: LoadError) -> Self {
        ConversionLoadError::Load(e)
    }
}

// ============================================================================
// Plugin
// ============================================================================

fn init_conversion_registry(mut commands: Commands) {
    let registry = match ConversionRegistry::load_from_file("content/attributes/conversions.ron") {
        Ok(reg) => {
            let errors = reg.validate();
            if !errors.is_empty() {
                for err in &errors {
                    bevy::log::warn!(target: "core", "ConversionRegistry validation: {}", err);
                }
            }
            reg
        }
        Err(e) => {
            bevy::log::error!(
                target: "core",
                error = %e,
                "Failed to load ConversionRegistry, using defaults"
            );
            ConversionRegistry::default()
        }
    };
    commands.insert_resource(registry);
}

/// 属性转换注册表 Plugin
pub struct ConversionRegistryPlugin;

impl Plugin for ConversionRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            init_conversion_registry.in_set(RegistryInitStage::Layer1),
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_calculate_ratio() {
        let conv = AttributeConversion {
            source: AttributeId::new("phys_def"),
            target: AttributeId::new("phys_atk"),
            ratio: 5000, // 50%
            condition: None,
        };
        assert_eq!(conv.calculate(100), 50);
        assert_eq!(conv.calculate(60), 30);
    }

    #[test]
    fn conversion_calculate_zero_source() {
        let conv = AttributeConversion {
            source: AttributeId::new("phys_def"),
            target: AttributeId::new("phys_atk"),
            ratio: 5000,
            condition: None,
        };
        assert_eq!(conv.calculate(0), 0);
    }

    #[test]
    fn conversion_calculate_full_ratio() {
        let conv = AttributeConversion {
            source: AttributeId::new("phys_def"),
            target: AttributeId::new("phys_atk"),
            ratio: 10000, // 100%
            condition: None,
        };
        assert_eq!(conv.calculate(100), 100);
    }

    #[test]
    fn registry_empty_by_default() {
        let reg = ConversionRegistry::default();
        assert!(reg.is_empty());
    }

    #[test]
    fn validation_rejects_non_positive_ratio() {
        let mut reg = ConversionRegistry::default();
        let source = AttributeId::new("phys_def");
        reg.conversions
            .entry(source.clone())
            .or_default()
            .push(AttributeConversion {
                source: source.clone(),
                target: AttributeId::new("phys_atk"),
                ratio: 0,
                condition: None,
            });
        let errors = reg.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("positive")));
    }

    #[test]
    fn validation_rejects_self_conversion() {
        let mut reg = ConversionRegistry::default();
        reg.conversions
            .entry(AttributeId::new("same"))
            .or_default()
            .push(AttributeConversion {
                source: AttributeId::new("same"),
                target: AttributeId::new("same"),
                ratio: 5000,
                condition: None,
            });
        let errors = reg.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("same")));
    }
}
