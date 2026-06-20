//! Condition 值对象定义
//!
//! 核心数据结构：递归 Condition 枚举直接建模领域条件树，
//! 支持 TagRequirement/AttributeCheck/ResourceCheck 三种叶子条件
//! 和 And/Or/Not 三种组合运算。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::capabilities::condition::foundation::types::{
    ComparisonOp, CustomConditionId, TagRequirementMode,
};
use crate::core::capabilities::tag::foundation::BitMask;
use crate::core::capabilities::tag::mechanism::query::InheritedMaskMap;

/// 条件树节点——递归枚举直接建模条件组合。
///
/// 领域规则 §1 定义的组合结构：
/// ```text
/// Condition (AND)
///   ├── TagRequirement: ...
///   ├── ConditionGroup (OR)
///   │    ├── AttributeCheck: ...
///   │    └── AttributeCheck: ...
///   └── NOT
///        └── ResourceCheck: ...
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    /// 基于标签的存在性/排除性检查。
    TagRequirement {
        /// 匹配模式（Has / Not）
        mode: TagRequirementMode,
        /// 目标标签 ID
        tag_id: String,
    },
    /// 基于 TagQuery 的多标签匹配（支持 Any/All/None + 层级继承）。
    TagMatch {
        /// 查询条件
        query: crate::core::capabilities::tag::foundation::TagQuery,
    },
    /// 基于属性阈值的数值检查。
    AttributeCheck {
        /// 属性 ID
        attribute_id: String,
        /// 比较运算符
        operator: ComparisonOp,
        /// 阈值
        threshold: f32,
    },
    /// 基于资源充足性的检查。
    ResourceCheck {
        /// 资源属性 ID
        resource_id: String,
        /// 所需最小量
        required_amount: f32,
    },
    /// 所有子条件通过则通过（AND）。
    And(Vec<Condition>),
    /// 任一子条件通过则通过（OR）。
    Or(Vec<Condition>),
    /// 子条件取反（NOT）。
    Not(Box<Condition>),
    /// 自定义条件（领域扩展点）。
    Custom(CustomCondition),
}

/// 自定义条件——领域扩展点。
///
/// 允许 Domain 注册特定条件逻辑，通过 id 分派到对应的外部检查函数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomCondition {
    /// 自定义条件标识
    pub id: CustomConditionId,
    /// 自定义参数（键值对，语义由注册方定义）
    pub params: HashMap<String, String>,
}

impl CustomCondition {
    /// 参数 HashMap 初始为空，由注册方在评估时填充查询所需键值。
    pub fn new(id: CustomConditionId) -> Self {
        Self {
            id,
            params: HashMap::new(),
        }
    }

    /// params 键值对语义由 CustomConditionId 对应的领域注册方定义（如 "minimum_level" → "5"）。
    pub fn with_params(id: CustomConditionId, params: HashMap<String, String>) -> Self {
        Self { id, params }
    }
}

/// 条件评估上下文——传递给评估器的实体状态快照。
#[derive(Debug, Clone)]
pub struct ConditionContext {
    /// 实体当前持有的标签 ID 列表。
    /// None 表示无法访问标签信息（标记为不通过）。
    pub tag_ids: Option<Vec<String>>,
    /// 实体当前的标签位掩码（用于 TagQuery 评估）。
    pub tag_bits: BitMask,
    /// TagId → BitMask 映射（由 TagHierarchy 维护）。
    pub tag_masks: Option<InheritedMaskMap>,
    /// 实体当前属性值（attribute_id → value）。
    pub attribute_values: HashMap<String, f32>,
}

impl ConditionContext {
    /// 创建空的评估上下文（所有检查均视为不通过）。
    pub fn empty() -> Self {
        Self {
            tag_ids: None,
            tag_bits: 0,
            tag_masks: None,
            attribute_values: HashMap::new(),
        }
    }

    /// 创建仅含属性的上下文（标签检查视为不通过）。
    pub fn with_attributes(values: HashMap<String, f32>) -> Self {
        Self {
            tag_ids: None,
            tag_bits: 0,
            tag_masks: None,
            attribute_values: values,
        }
    }

    /// 标签 ID 列表用于 TagRequirement 检查。tag_bits 为 0，TagQuery 评估将不通过。
    pub fn with_tags(tag_ids: Vec<String>) -> Self {
        Self {
            tag_ids: Some(tag_ids),
            tag_bits: 0,
            tag_masks: None,
            attribute_values: HashMap::new(),
        }
    }

    /// 创建带位掩码标签的上下文（用于 TagQuery 评估）。
    pub fn with_tag_bits(tag_bits: BitMask, tag_masks: InheritedMaskMap) -> Self {
        Self {
            tag_ids: None,
            tag_bits,
            tag_masks: Some(tag_masks),
            attribute_values: HashMap::new(),
        }
    }
}
