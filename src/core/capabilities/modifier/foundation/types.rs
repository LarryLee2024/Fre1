crate::define_numeric_id!(ModifierInstanceId);

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
