//! C2: 规则与系统层 — ECS 组件、管线执行器、生命周期

mod components;
pub(crate) mod pipeline;

mod lifecycle;
pub(crate) mod systems;

pub use components::*;
pub use lifecycle::*;
