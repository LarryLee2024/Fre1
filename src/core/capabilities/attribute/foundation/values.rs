use bevy::asset::Asset;
use bevy::prelude::Reflect;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::core::capabilities::attribute::foundation::types::*;

/// 属性的静态定义（运行时只读）。
///
/// 在内容加载阶段构建，运行时不可修改。
/// 不变量：
/// - default_base_value ∈ [min_value, max_value]
/// - Resource 类属性的 min_value ≥ 0
#[derive(Debug, Clone, Asset, Serialize, Deserialize, TypePath)]
pub struct AttributeDefinition {
    /// 属性唯一标识符
    pub id: AttributeId,
    /// 属性分类，决定聚合管线处理方式
    pub category: AttributeCategory,
    /// 初始 base 值
    pub default_base_value: f32,
    /// 属性下限，聚合后 Clamp 阶段使用
    pub min_value: f32,
    /// 属性上限，聚合后 Clamp 阶段使用
    pub max_value: f32,
    /// Derived 类属性依赖的其他属性 ID 列表
    pub derived_dependencies: Vec<AttributeId>,
    /// 是否在 UI 中隐藏（调试/内部属性用）
    pub hidden: bool,
}

/// 属性的运行时数值。
///
/// 不变量：
/// - current_value 由 Aggregator 管线计算，不允许外部直接修改
/// - base_value 由内容初始化或 Modifier Override 阶段写入
#[derive(Debug, Clone, Reflect)]
pub struct AttributeValue {
    /// 关联的静态定义 ID
    pub def_id: AttributeId,
    /// 基础值（不含 Modifier 的原始值）
    pub base_value: f32,
    /// 聚合后的当前值（含 Modifier + Clamp）
    pub current_value: f32,
    /// 是否由 Aggregator 管线自动管理（true 时外部不可直接写 current_value）
    pub aggregator_managed: bool,
}

/// 派生属性计算公式。
///
/// 用于 Derived 类属性从其他属性推算最终值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedFormula {
    /// 目标属性 ID（必须已注册）
    pub target_attr_id: AttributeId,
    /// 公式类型
    pub formula_type: FormulaType,
    /// 公式参数（不同 FormulaType 使用不同字段）
    pub parameters: FormulaParameters,
}

/// 公式类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormulaType {
    /// 常量值，忽略所有依赖属性
    Constant,
    /// 求和，将所有 source_ids 对应属性值相加
    Sum,
    /// 取最大值
    Max,
    /// 取最小值
    Min,
    /// 加权求和，按 weights 中的权重乘以各属性值后求和
    WeightedSum,
    /// 自定义公式，由 formula_id 引用外部计算逻辑
    Custom { formula_id: String },
}

/// 公式参数。
///
/// 不同 FormulaType 使用不同的参数字段，未使用的字段为 None。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaParameters {
    /// Constant 类型使用的常量值
    pub constant: Option<f32>,
    /// Sum/Max/Min 类型使用的源属性 ID 列表
    pub source_ids: Option<Vec<AttributeId>>,
    /// WeightedSum 类型的全局乘数
    pub multiplier: Option<f32>,
    /// WeightedSum 类型的各属性权重 (AttributeId, weight)
    pub weights: Option<Vec<(AttributeId, f32)>>,
    /// WeightedSum 类型的基础偏移量
    pub base: Option<f32>,
    /// Custom 类型的外部公式标识符
    pub formula_id: Option<String>,
}
