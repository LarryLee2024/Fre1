//! Condition 基础类型与枚举
//!
//! 包含条件类型分类、逻辑运算、比较运算符和评估结果类型。

use serde::{Deserialize, Serialize};

/// Condition 逻辑组合运算。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConditionOp {
    /// 所有子条件通过则通过（短路：任一失败即返回 Fail）
    And,
    /// 任一子条件通过则通过（短路：任一通过即返回 Pass）
    Or,
    /// 子条件取反
    Not,
}

/// 数值比较运算符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 大于
    GreaterThan,
    /// 大于等于
    GreaterOrEqual,
    /// 小于
    LessThan,
    /// 小于等于
    LessOrEqual,
}

impl ComparisonOp {
    /// Equal/NotEqual 使用 epsilon 比较以避免浮点精度问题，其余运算符直接比较。
    pub fn evaluate(&self, actual: f32, threshold: f32) -> bool {
        match self {
            Self::Equal => (actual - threshold).abs() < f32::EPSILON,
            Self::NotEqual => (actual - threshold).abs() >= f32::EPSILON,
            Self::GreaterThan => actual > threshold,
            Self::GreaterOrEqual => actual >= threshold,
            Self::LessThan => actual < threshold,
            Self::LessOrEqual => actual <= threshold,
        }
    }
}

/// 标签匹配模式（用于 TagRequirement）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagRequirementMode {
    /// 目标必须持有指定标签
    Has,
    /// 目标必须不持有指定标签（用于免疫/排除检查）
    Not,
}

/// 条件评估结果。
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionResult {
    /// 条件通过
    Passed,
    /// 条件不通过，附带失败原因
    Failed { reason: String },
}

impl ConditionResult {
    /// 与 matches!(result, ConditionResult::Passed) 等价，提供更可读的调用方式。
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }

    /// 无参数快捷构造，用于条件树评估成功时的返回。
    pub fn passed() -> Self {
        Self::Passed
    }

    /// 失败时要求附带原因，由评估器在短路时提供领域可读的错误信息。
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::Failed {
            reason: reason.into(),
        }
    }
}

/// 自定义条件标识（由具体领域定义并注册）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomConditionId(pub u32);
