//! C2: 规则与系统层 — ECS 组件、查询、生命周期、System

mod components;
pub mod lifecycle;
pub mod query;
pub mod systems;

pub use components::*;
pub use lifecycle::*;
pub use query::*;
