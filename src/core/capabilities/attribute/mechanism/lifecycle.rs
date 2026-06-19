use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeDefinition, AttributeId, DerivedFormula,
};

/// 全局属性注册表（Resource）。
/// 在内容加载阶段构建，运行时只读。
#[derive(Resource, Debug, Clone, Default)]
pub struct AttributeRegistry {
    pub definitions: HashMap<AttributeId, AttributeDefinition>,
    pub formulas: HashMap<AttributeId, DerivedFormula>,
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
        if def.category == AttributeCategory::Derived
            && self.would_create_cycle(&def.id, &def.derived_dependencies)
        {
            return Err(AttributeRegistrationError::DerivedCircularDependency(
                def.id,
            ));
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
