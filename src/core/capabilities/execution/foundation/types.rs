//! Execution 基础类型与枚举
//!
//! 定义执行计算类型、直接操作类型。
//!
//! 详见 docs/02-domain/capabilities/execution_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/execution_schema.md §3。

use serde::{Deserialize, Serialize};

use crate::core::capabilities::execution::foundation::values::DamageParams;
use crate::core::capabilities::execution::foundation::values::HealParams;

/// 执行计算类型枚举，定义计算的业务类别。
///
/// 分为两大类：内置计算（Damage/Heal/DirectAttributeMod）和扩展计算（Custom）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// 返回的字符串与 ExecutionType 变体名一致，用于日志和运行时类型分发。
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                let levels_above_base = level.saturating_sub(1);
                base + per_level * levels_above_base as f32
            }
        }
    }
}

/// 自定义执行引用——指向 Domains 注册的自定义计算逻辑。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomExecutionRef {
    /// 自定义执行 ID（对应 CustomExecutionRegistry 中的注册项）
    pub execution_id: String,
    /// 自定义参数（传递给自定义计算的领域特定数据）
    pub params: std::collections::HashMap<String, String>,
}

impl CustomExecutionRef {
    /// execution_id 指向 CustomExecutionRegistry 中已注册的计算逻辑。params 初始为空。
    pub fn new(execution_id: impl Into<String>) -> Self {
        Self {
            execution_id: execution_id.into(),
            params: std::collections::HashMap::new(),
        }
    }

    /// 参数语义由 execution_id 对应的注册方定义（如 "damage_scale" → "1.5"）。
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

