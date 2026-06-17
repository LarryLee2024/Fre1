//! rules — Tactical 域纯业务规则（零 ECS 依赖）
//!
//! 所有移动消耗、范围计算的纯函数。
//! 详见 docs/02-domain/domains/tactical_domain.md §5

pub(crate) mod movement;
pub(crate) mod range;
