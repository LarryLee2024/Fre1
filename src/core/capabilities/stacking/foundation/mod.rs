//! Stacking Foundation — 堆叠规则基础类型与值对象

pub(crate) mod types;
pub(crate) mod values;

pub use types::{
    OverflowBehavior, StackIdentity, StackMatchResult, StackingConfig, StackingDecision,
    StackingError, StackingType,
};
pub use values::StackingState;
