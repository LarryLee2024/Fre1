use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

crate::define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

/// 属性分类枚举，决定属性在聚合管线中的处理方式。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeCategory {
    /// 基础属性（如 ATK、DEF），由内容直接定义
    Primary,
    /// 次级属性（如 CRIT_RATE），可由 Primary 推算
    Secondary,
    /// 派生属性（如 FINAL_ATK），由公式从其他属性计算
    Derived,
    /// 资源属性（如 HP、MP），受 Modifier 管线约束且 min ≥ 0
    Resource,
}
