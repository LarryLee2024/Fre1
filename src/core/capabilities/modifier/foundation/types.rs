use std::fmt;

/// 修改器实例唯一标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierInstanceId(pub u64);

impl ModifierInstanceId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for ModifierInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mod_inst_{}", self.0)
    }
}

/// 修改器运算类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierOp {
    Add,
    Multiply,
    Override,
}

/// 修改器执行优先级（越小越优先）。
pub type ModifierPriority = u8;

/// 修改器来源类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierSourceType {
    Buff,
    Equipment,
    Ability,
    Passive,
    Environmental,
    Item,
    Progression,
    Custom(String),
}
