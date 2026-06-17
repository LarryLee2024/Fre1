//! Execution 基础类型与枚举
//!
//! 定义执行计算类型、直接操作类型以及领域错误。
//!
//! 详见 docs/02-domain/execution_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/execution_schema.md §3。

use crate::core::capabilities::execution::foundation::values::DamageParams;
use crate::core::capabilities::execution::foundation::values::HealParams;

/// 执行计算类型枚举，定义计算的业务类别。
///
/// 分为两大类：内置计算（Damage/Heal/DirectAttributeMod）和扩展计算（Custom）。
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionType {
    /// 伤害计算
    Damage(DamageParams),
    /// 治疗计算
    Heal(HealParams),
    /// 自定义计算（扩展点，由 Domains 注册）
    Custom(CustomExecutionRef),
    /// 直接修改属性（如设置某属性为固定值）
    DirectAttributeMod {
        /// 属性标识
        attribute_id: String,
        /// 操作类型
        operation: DirectOp,
        /// 值
        value: ScalableValue,
    },
    /// 空执行（什么都不做，用于占位）
    None,
}

impl ExecutionType {
    /// 返回人类可读的执行类型名。
    pub fn name(&self) -> &str {
        match self {
            Self::Damage(_) => "Damage",
            Self::Heal(_) => "Heal",
            Self::Custom(_) => "Custom",
            Self::DirectAttributeMod { .. } => "DirectAttributeMod",
            Self::None => "None",
        }
    }

    /// 是否为数值计算类型（Damage 或 Heal）。
    pub fn is_numeric_calculation(&self) -> bool {
        matches!(self, Self::Damage(_) | Self::Heal(_))
    }
}

/// 直接属性修改操作枚举。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectOp {
    /// 设置为固定值
    Set,
    /// 增加
    Add,
    /// 减少
    Subtract,
    /// 乘以系数
    Multiply,
}

/// 可缩放值类型，定义可以随等级/属性缩放的值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScalableValue {
    /// 固定值
    Fixed(f32),
    /// 按等级缩放（基础值 + 每级增量）
    PerLevel {
        /// 基础值（等级 1）
        base: f32,
        /// 每级增量
        per_level: f32,
    },
}

impl ScalableValue {
    /// 计算在指定等级下的实际值。
    ///
    /// 等级从 1 开始索引。
    pub fn calculate(&self, level: u32) -> f32 {
        match self {
            Self::Fixed(value) => *value,
            Self::PerLevel { base, per_level } => {
                let levels_above_base = if level > 1 { level - 1 } else { 0 };
                base + per_level * levels_above_base as f32
            }
        }
    }
}

/// 自定义执行引用——指向 Domains 注册的自定义计算逻辑。
#[derive(Debug, Clone, PartialEq)]
pub struct CustomExecutionRef {
    /// 自定义执行 ID（对应 CustomExecutionRegistry 中的注册项）
    pub execution_id: String,
    /// 自定义参数（传递给自定义计算的领域特定数据）
    pub params: std::collections::HashMap<String, String>,
}

impl CustomExecutionRef {
    /// 创建新的自定义执行引用。
    pub fn new(execution_id: impl Into<String>) -> Self {
        Self {
            execution_id: execution_id.into(),
            params: std::collections::HashMap::new(),
        }
    }

    /// 添加参数。
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// Execution 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    /// 计算公式 ID 未注册（V1: formula_id 已注册）
    FormulaNotFound { formula_id: String, detail: String },
    /// ExecutionContext 数据缺失（不变量 3.3）
    ContextMissing { field: String, detail: String },
    /// 计算结果数值非法（不变量 3.4）
    InvalidResult(String),
    /// 自定义计算未注册（不变量 3.5）
    CustomExecutionNotRegistered(String),
    /// 不支持的执行类型
    UnsupportedExecutionType(String),
    /// 通用运行时错误
    Runtime(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FormulaNotFound { formula_id, detail } => {
                write!(f, "formula '{}' not found: {}", formula_id, detail)
            }
            Self::ContextMissing { field, detail } => {
                write!(f, "context field '{}' missing: {}", field, detail)
            }
            Self::InvalidResult(msg) => write!(f, "invalid result: {}", msg),
            Self::CustomExecutionNotRegistered(id) => {
                write!(f, "custom execution '{}' not registered", id)
            }
            Self::UnsupportedExecutionType(msg) => {
                write!(f, "unsupported execution type: {}", msg)
            }
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}
