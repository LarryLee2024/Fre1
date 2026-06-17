//! rules — Terrain 域纯业务规则（零 ECS 依赖）
//!
//! 所有地形移动消耗、遮蔽度计算的纯函数。
//! 详见 docs/02-domain/domains/terrain_domain.md §5

pub(crate) mod concealment;
pub(crate) mod movement_cost;
