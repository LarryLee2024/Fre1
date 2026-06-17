//! Stacking Foundation — 堆叠规则基础类型与值对象

pub mod types;
pub mod values;

pub use types::{
    OverflowBehavior, StackIdentity, StackMatchResult, StackingConfig, StackingDecision,
    StackingError, StackingType,
};
pub use values::StackingState;
