//! 跨 crate 使用的共享常量。
//!
//! 本模块包含具有零业务或技术语义的全局常量 — 它们是跨层共享的纯配置值。
//! 领域特定的常量请参见各自的领域模块。

/// Observer 链式反应的最大深度。
///
/// 当 Observer 递归触发其他 Observer 时，此限制防止无限循环。
/// 基于 ADR-002 建议设置为 10。
///
/// 运行时违规触发 WARN 日志但不会 panic。
pub const MAX_OBSERVER_DEPTH: u32 = 10;

/// 最大队伍成员数。
pub const MAX_PARTY_SIZE: usize = 6;

/// 每个实体的最大背包槽位数。
pub const MAX_INVENTORY_SIZE: usize = 100;

/// 默认最大 Buff 叠加层数。
pub const MAX_BUFF_STACK: usize = 5;
