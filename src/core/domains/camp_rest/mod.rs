//! camp_rest — 营地/休息业务领域
//!
//! 管理短休、长休、生命骰、营地事件。
//! 详见 docs/02-domain/domains/camp_rest_domain.md
//! 详见 ADR-031

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误定义，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 领域事件定义，crate 内共享
pub(crate) mod events;
// [ADR-045] pub(crate) — 业务规则失败定义，crate 内共享
pub(crate) mod failure;
// [ADR-045] pub(crate) — 集成层，外部访问 CampRest 组件的唯一入口
pub(crate) mod integration;
mod plugin;
mod resources;
mod rules;
mod systems;

pub use components::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;
