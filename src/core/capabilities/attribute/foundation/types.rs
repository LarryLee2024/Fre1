use std::fmt;

/// 属性唯一标识符，强类型 newtype。
/// 格式: `attr_<6位数字>`（如 `attr_000001`）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeId(pub String);

impl AttributeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AttributeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 属性分类枚举。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeCategory {
    Primary,
    Secondary,
    Derived,
    Resource,
}
