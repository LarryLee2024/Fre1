use bevy::asset::Asset;
use bevy::prelude::Reflect;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::core::capabilities::attribute::foundation::types::*;

/// 属性的静态定义（运行时只读）。
#[derive(Debug, Clone, Asset, Serialize, Deserialize, TypePath)]
pub struct AttributeDefinition {
    pub id: AttributeId,
    pub category: AttributeCategory,
    pub default_base_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub derived_dependencies: Vec<AttributeId>,
    pub hidden: bool,
}

/// 属性的运行时数值。
#[derive(Debug, Clone, Reflect)]
pub struct AttributeValue {
    pub def_id: AttributeId,
    pub base_value: f32,
    pub current_value: f32,
    pub aggregator_managed: bool,
}

/// 派生属性计算公式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedFormula {
    pub target_attr_id: AttributeId,
    pub formula_type: FormulaType,
    pub parameters: FormulaParameters,
}

/// 公式类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormulaType {
    Constant,
    Sum,
    Max,
    Min,
    WeightedSum,
    Custom { formula_id: String },
}

/// 公式参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaParameters {
    pub constant: Option<f32>,
    pub source_ids: Option<Vec<AttributeId>>,
    pub multiplier: Option<f32>,
    pub weights: Option<Vec<(AttributeId, f32)>>,
    pub base: Option<f32>,
    pub formula_id: Option<String>,
}
