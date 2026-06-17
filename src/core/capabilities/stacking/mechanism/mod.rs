//! Stacking Mechanism — 堆叠判定与逻辑

pub(crate) mod decider;

pub use decider::{
    StackingOutcome, StackingSubject, decide_stacking, evaluate_stacking, match_identity,
    validate_config,
};
