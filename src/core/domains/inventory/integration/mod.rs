//! integration — Inventory 域对外访问接口层。
//!
//! 此模块是其他域查询/修改 Inventory 域数据的唯一入口。
//! 按功能拆分为：
//!
//! - `facade` — InventoryReadFacade（只读查询）+ InventoryWriteFacade（写入修改）
//! - `query` — InventoryQueryParam（Bevy SystemParam 封装）
//!
//! # 设计原则
//!
//! 1. Systems 通过 SystemParam 或 Facade 访问 Inventory 数据，不直接引用组件
//! 2. Facade 只做纯数据访问，不包含业务逻辑
//! 3. 当 Inventory 组件结构变化时，只需修改此处
//!
//! # 参考
//!
//! - `docs/01-architecture/README.md` §6.2
//! - `docs/01-architecture/30-progression-narrative/ADR-030-progression-inventory.md`
//! - `docs/02-domain/domains/inventory_domain.md`

pub mod facade;
pub mod query;

pub use facade::*;
pub use query::InventoryQueryParam;
