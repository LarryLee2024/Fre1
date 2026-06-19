//! integration — Combat 域与 Capabilities 的 Anti-Corruption Layer。
//!
//! 此模块是 Combat 域调用 Capabilities 的唯一入口。
//! 按能力域拆分为子模块，避免 God File 膨胀：
//!
//! ## Phase E (已接入)
//! - `effect/` — 效果能力（Effect）—— Effect Tick、效果查询
//! - `ability/` — 技能能力（Ability）—— 技能激活、冷却管理
//! - `condition/` — 条件能力（Condition）—— 免疫检查、施法条件
//! - `trigger/` — 触发器能力（Trigger）—— 战斗事件触发器评估
//! - `event/` — 事件能力（Event）—— EventBus 统一事件分发
//! - `turn/` — 回合管理
//!
//! ## Phase F (已接入)
//! - `targeting/` — 目标选择能力（Targeting）—— 战斗目标筛选与校验
//! - `execution/` — 执行计算能力（Execution）—— 伤害/治疗数值结算
//! - `gameplay_context/` — 上下文能力（GameplayContext）—— 战斗全链路上下文
//! - `aggregator/` — 属性聚合能力（Aggregator）—— Modifier 聚合重算
//!
//! # 设计原则
//!
//! 1. Systems 通过 SystemParam + View Types 交互，不知道 Capabilities 内部类型
//! 2. Facade 函数是唯一访问 Capabilities 字段的地方
//! 3. 当 Capabilities 内部结构变化时，只修改此处 facade.rs
//!
//! # 参考
//!
//! - `docs/01-architecture/README.md` §6.2 — Business Domain 标准结构
//! - `docs/01-architecture/20-tactical-combat/ADR-024-combat-integration-layer.md`
//! - `docs/02-domain/domains/combat_domain.md` §7
//!
//! 🟥 禁止任何 Systems 直接 import Capabilities 组件类型进行字段访问。

pub mod ability;
pub mod aggregator;
pub mod condition;
pub mod effect;
pub mod event;
pub mod execution;
pub mod gameplay_context;
pub mod replay;
pub mod targeting;
pub mod trigger;
pub mod turn;
