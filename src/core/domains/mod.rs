//! Domains — 15 个业务子系统
//!
//! 承载全部玩法复杂度，每个领域独立演进。
//! 标准内部结构: plugin.rs + components.rs + systems/ + events.rs + error.rs + rules/ + integration.rs
//!
//! 详见 `docs/01-architecture/README.md` §3.3

pub mod camp_rest;
pub mod combat;
pub mod crafting;
pub mod economy;
pub mod faction;
pub mod inventory;
pub mod narrative;
pub mod party;
pub mod progression;
pub mod quest;
pub mod reaction;
pub mod spell;
pub mod summon;
pub mod tactical;
pub mod terrain;
