//! integration — Tactical 域与 Capabilities 的 Anti-Corruption Layer。
//!
//! 此模块是 Tactical 域调用 Capabilities 的唯一入口。
//! 按能力域拆分为子模块，避免 God File 膨胀：
//!
//! - `movement/` — 移动能力（Tag/Attribute/Modifier 查询与写入）
//! - `command_handler` — GameCommand 路由转领域事件
//! - 未来：`terrain/`, `targeting/`, `vision/`, `attack/`
//!
//! 设计原则：
//! 1. Systems 通过 SystemParam + View Types 交互，不知道 Capabilities 内部类型
//! 2. Facade 函数是唯一访问 Capabilities 字段的地方
//! 3. 当 Capabilities 内部结构变化时，只修改此处 facade.rs
//!
//! 详见 docs/02-domain/domains/tactical_domain.md §7
//! 详见 docs/01-architecture/00-foundation/ADR-022

mod command_handler;
pub mod movement;

pub use command_handler::on_tactical_command;
