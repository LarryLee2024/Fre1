//! economy — 经济/交易业务领域
//!
//! 管理货币系统（多币种钱包）、交易流程（购买/出售/补货）、商店库存与供需系统。
//! 集成 reputation/supply/stolen 价格修正机制（当前骨架阶段简化处理）。
//! 详见 docs/02-domain/domains/economy_domain.md

mod components;
mod error;
mod events;
mod failure;
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
