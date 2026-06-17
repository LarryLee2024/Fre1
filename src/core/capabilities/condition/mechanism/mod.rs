//! C2: 规则与系统层 — Condition ECS 组件、评估器、System

mod components;
mod evaluator;
pub(crate) mod systems;

pub use components::*;
pub use evaluator::{check_immunity, evaluate};
