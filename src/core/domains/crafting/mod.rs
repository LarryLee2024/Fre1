//! crafting — 制作/锻造业务领域
//!
//! 管理配方学习与解锁、材料消耗、装备附魔、武器/防具升级路线。
//! 集成 crafting station 交互流程与品质判定。
//! 详见 docs/02-domain/domains/crafting_domain.md

mod components;
mod error;
mod events;
mod plugin;
mod resources;
mod rules;
mod systems;

pub use components::*;
pub use error::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;

#[cfg(test)]
mod tests;
