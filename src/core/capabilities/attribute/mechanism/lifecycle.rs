use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeDefinition, AttributeId, DerivedFormula,
};

/// 全局属性注册表（Resource）。
/// 在内容加载阶段构建，运行时只读。
#[derive(Resource, Debug, Clone)]
pub struct AttributeRegistry {
    pub definitions: HashMap<AttributeId, AttributeDefinition>,
    pub formulas: HashMap<AttributeId, DerivedFormula>,
}

impl Default for AttributeRegistry {
    fn default() -> Self {
        Self {
            definitions: HashMap::new(),
            formulas: HashMap::new(),
        }
    }
}

/// 属性注册错误。
#[derive(Debug)]
pub enum AttributeRegistrationError {
    DuplicateId(AttributeId),
    DefaultValueOutOfRange {
        id: AttributeId,
        value: f32,
        min: f32,
        max: f32,
    },
    DerivedDependencyNotFound {
        attr_id: AttributeId,
        missing_dep: AttributeId,
    },
    DerivedCircularDependency(AttributeId),
    ResourceMinBelowZero(AttributeId),
    FormulaTargetNotFound(AttributeId),
}

impl std::fmt::Display for AttributeRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate attribute ID: {}", id),
            Self::DefaultValueOutOfRange {
                id,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "attribute {} default value {} out of range [{}, {}]",
                    id, value, min, max
                )
            }
            Self::DerivedDependencyNotFound {
                attr_id,
                missing_dep,
            } => {
                write!(
                    f,
                    "derived attribute {} depends on missing attribute {}",
                    attr_id, missing_dep
                )
            }
            Self::DerivedCircularDependency(id) => {
                write!(
                    f,
                    "circular dependency detected for derived attribute: {}",
                    id
                )
            }
            Self::ResourceMinBelowZero(id) => {
                write!(f, "resource attribute {} has min_value < 0", id)
            }
            Self::FormulaTargetNotFound(id) => {
                write!(f, "formula target attribute {} not found in registry", id)
            }
        }
    }
}

impl std::error::Error for AttributeRegistrationError {}

impl AttributeRegistry {
    /// 注册单个属性定义，执行全部校验。
    pub fn register(&mut self, def: AttributeDefinition) -> Result<(), AttributeRegistrationError> {
        // V1: AttributeId 全局唯一
        if self.definitions.contains_key(&def.id) {
            return Err(AttributeRegistrationError::DuplicateId(def.id));
        }

        // V2: 默认 base 值在 [min, max] 范围
        if def.default_base_value < def.min_value || def.default_base_value > def.max_value {
            return Err(AttributeRegistrationError::DefaultValueOutOfRange {
                id: def.id.clone(),
                value: def.default_base_value,
                min: def.min_value,
                max: def.max_value,
            });
        }

        // V5: Resource 属性 min ≥ 0
        if def.category == AttributeCategory::Resource && def.min_value < 0.0 {
            return Err(AttributeRegistrationError::ResourceMinBelowZero(
                def.id.clone(),
            ));
        }

        // V3: Derived 公式引用的属性必须已注册
        for dep in &def.derived_dependencies {
            if !self.definitions.contains_key(dep) {
                return Err(AttributeRegistrationError::DerivedDependencyNotFound {
                    attr_id: def.id.clone(),
                    missing_dep: dep.clone(),
                });
            }
        }

        // V4: 无循环引用（限制在 Derived 类别）
        if def.category == AttributeCategory::Derived {
            if self.would_create_cycle(&def.id, &def.derived_dependencies) {
                return Err(AttributeRegistrationError::DerivedCircularDependency(
                    def.id,
                ));
            }
        }

        let id = def.id.clone();
        self.definitions.insert(id, def);
        Ok(())
    }

    /// 注册派生属性公式。
    pub fn register_formula(
        &mut self,
        formula: DerivedFormula,
    ) -> Result<(), AttributeRegistrationError> {
        if !self.definitions.contains_key(&formula.target_attr_id) {
            return Err(AttributeRegistrationError::FormulaTargetNotFound(
                formula.target_attr_id,
            ));
        }
        let id = formula.target_attr_id.clone();
        self.formulas.insert(id, formula);
        Ok(())
    }

    /// 批量注册属性定义。
    pub fn register_batch(
        &mut self,
        defs: Vec<AttributeDefinition>,
    ) -> Vec<Result<(), AttributeRegistrationError>> {
        defs.into_iter().map(|def| self.register(def)).collect()
    }

    /// DFS 检测循环引用: A → B → A
    fn would_create_cycle(&self, attr_id: &AttributeId, deps: &[AttributeId]) -> bool {
        let mut visited = HashSet::new();
        let mut stack = deps.to_vec();
        while let Some(next) = stack.pop() {
            if &next == attr_id {
                return true;
            }
            if !visited.insert(next.clone()) {
                continue;
            }
            if let Some(def) = self.definitions.get(&next) {
                stack.extend(def.derived_dependencies.iter().cloned());
            }
        }
        false
    }

    pub fn get(&self, id: &AttributeId) -> Option<&AttributeDefinition> {
        self.definitions.get(id)
    }

    pub fn contains(&self, id: &AttributeId) -> bool {
        self.definitions.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attr(
        id: &str,
        category: AttributeCategory,
        default: f32,
        min: f32,
        max: f32,
        deps: Vec<&str>,
    ) -> AttributeDefinition {
        AttributeDefinition {
            id: AttributeId::new(id),
            category,
            default_base_value: default,
            min_value: min,
            max_value: max,
            derived_dependencies: deps.into_iter().map(AttributeId::new).collect(),
            hidden: false,
        }
    }

    #[test]
    fn unit_001_register_primary_attr() {
        let mut reg = AttributeRegistry::default();
        let def = make_attr(
            "attr_000001",
            AttributeCategory::Primary,
            10.0,
            1.0,
            30.0,
            vec![],
        );
        assert!(reg.register(def).is_ok());
        assert_eq!(reg.definitions.len(), 1);
    }

    #[test]
    fn unit_002_duplicate_id_rejected() {
        let mut reg = AttributeRegistry::default();
        reg.register(make_attr(
            "attr_000001",
            AttributeCategory::Primary,
            10.0,
            1.0,
            30.0,
            vec![],
        ))
        .unwrap();
        let result = reg.register(make_attr(
            "attr_000001",
            AttributeCategory::Primary,
            10.0,
            1.0,
            30.0,
            vec![],
        ));
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::DuplicateId(_))
        ));
    }

    #[test]
    fn unit_003_default_value_out_of_range() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(make_attr(
            "attr_000001",
            AttributeCategory::Primary,
            50.0,
            1.0,
            30.0,
            vec![],
        ));
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::DefaultValueOutOfRange { .. })
        ));
    }

    #[test]
    fn unit_004_resource_min_below_zero() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(make_attr(
            "attr_000001",
            AttributeCategory::Resource,
            100.0,
            -10.0,
            100.0,
            vec![],
        ));
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::ResourceMinBelowZero(_))
        ));
    }

    #[test]
    fn unit_005_derived_dependency_not_found() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(make_attr(
            "attr_000010",
            AttributeCategory::Derived,
            50.0,
            0.0,
            100.0,
            vec!["attr_999999"],
        ));
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::DerivedDependencyNotFound { .. })
        ));
    }

    #[test]
    fn unit_006_register_formula() {
        let mut reg = AttributeRegistry::default();
        reg.register(make_attr(
            "attr_000001",
            AttributeCategory::Primary,
            10.0,
            1.0,
            30.0,
            vec![],
        ))
        .unwrap();
        reg.register(make_attr(
            "attr_000010",
            AttributeCategory::Derived,
            50.0,
            0.0,
            100.0,
            vec!["attr_000001"],
        ))
        .unwrap();
        let formula = DerivedFormula {
            target_attr_id: AttributeId::new("attr_000010"),
            formula_type: crate::core::capabilities::attribute::foundation::FormulaType::Sum,
            parameters: crate::core::capabilities::attribute::foundation::FormulaParameters {
                constant: None,
                source_ids: Some(vec![AttributeId::new("attr_000001")]),
                multiplier: Some(1.0),
                weights: None,
                base: None,
                formula_id: None,
            },
        };
        assert!(reg.register_formula(formula).is_ok());
    }

    #[test]
    fn unit_007_formula_target_not_found() {
        let mut reg = AttributeRegistry::default();
        let formula = DerivedFormula {
            target_attr_id: AttributeId::new("attr_999999"),
            formula_type: crate::core::capabilities::attribute::foundation::FormulaType::Constant,
            parameters: crate::core::capabilities::attribute::foundation::FormulaParameters {
                constant: Some(10.0),
                source_ids: None,
                multiplier: None,
                weights: None,
                base: None,
                formula_id: None,
            },
        };
        let result = reg.register_formula(formula);
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::FormulaTargetNotFound(_))
        ));
    }
}
