//! Capabilities — 15 个核心能力领域
//!
//! 贯穿所有游戏机制的"魔法圈" — 从 Tag 分类到 Effect 执行到 Cue 表现的完整链路。
//! 每个领域遵循 C1 Foundation → C2 Mechanism → C3 Runtime 自包含组织。
//!
//! 详见 `docs/01-architecture/README.md` §3.2

pub mod ability;
pub mod aggregator;
pub mod attribute;
pub mod condition;
pub mod cue;
pub mod effect;
pub mod event;
pub mod execution;
pub mod gameplay_context;
pub mod modifier;
pub mod rule;
pub mod runtime;
pub mod spec;
pub mod stacking;
pub mod tag;
pub mod targeting;
pub mod trigger;
