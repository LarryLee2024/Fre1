use bevy::prelude::Reflect;

crate::define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

/// 属性分类枚举。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeCategory {
    Primary,
    Secondary,
    Derived,
    Resource,
}
