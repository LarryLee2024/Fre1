//! rules — Faction 域纯业务规则（零 ECS 依赖）
//!
//! 所有声望计算、关系判定的纯函数。
//! 详见 docs/02-domain/domains/faction_domain.md §5

pub(crate) mod relationship;
pub(crate) mod reputation;
