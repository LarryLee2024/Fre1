//! Rule 基础类型与枚举
//!
//! 定义数据驱动规则的类型系统。
//!
//! 详见 docs/02-domain/capabilities/rule_domain.md。

use serde::{Deserialize, Serialize};

/// 规则效果类型——当条件满足时执行的动作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleEffect {
    /// 数值修改（通过 Modifier 管线）
    Modifier {
        /// 目标属性 ID
        target_attribute: String,
        /// 修改操作
        op: RuleModifierOp,
        /// 修改值
        value: f32,
        /// 优先级
        priority: u8,
    },
    /// 标签授予/移除
    TagOperation {
        /// 授予的标签
        grant: Vec<String>,
        /// 移除的标签
        remove: Vec<String>,
    },
    /// 事件触发
    EventTrigger {
        /// 事件类型标识
        event_type: String,
        /// 事件参数
        #[serde(default)]
        params: std::collections::HashMap<String, String>,
    },
    /// 效果应用
    EffectApply {
        /// 效果定义 ID
        effect_id: String,
    },
}

/// 规则内的修改器操作（简化版 ModifierOp，用于规则定义层）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleModifierOp {
    /// 加法
    Add,
    /// 乘法
    Multiply,
    /// 覆盖
    Override,
}
