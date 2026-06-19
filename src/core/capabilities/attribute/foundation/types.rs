use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

crate::define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

/// 属性分类枚举。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeCategory {
    Primary,
    Secondary,
    Derived,
    Resource,
}
