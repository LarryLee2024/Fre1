//! integration — Progression 域的 Anti-Corruption Layer。
//!
//! 此模块是外部访问 Progression 域组件的统一入口。
//! 调用方通过 ReadFacade / WriteFacade / QueryParam 交互，
//! 永远不直接 import Progression domain 的 Components。
//!
//! 设计原则：
//! 1. Systems 通过 SystemParam + View 交互，不知道 Progression 组件内部细节
//! 2. Facade 是唯一直接访问 Progression 组件字段的地方
//! 3. 当 Progression 组件结构变化时，只修改此模块
//!
//! 详见 docs/02-domain/domains/progression_domain.md §7
//! 详见 docs/01-architecture/README.md §6.2

pub mod facade;
pub mod query;

pub use facade::{ProgressionReadFacade, ProgressionWriteFacade};
pub use query::ProgressionQueryParam;
