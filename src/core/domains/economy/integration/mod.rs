//! integration — Economy 域的 Anti-Corruption Layer。
//!
//! 此模块是外部代码访问 Economy 域 ECS 组件的唯一入口。
//! 外部代码（其他 Domains 或 Capabilities）必须通过此模块的
//! ReadFacade / WriteFacade 读取或修改经济数据，禁止直接 import
//! Economy 域的 Component 类型。
//!
//! # 设计原则
//!
//! 1. Systems 通过 `EconomyQueryParam` (SystemParam) 或 Facade 的静态方法交互
//! 2. Facade 是唯一访问 Economy 域内部组件字段的地方
//! 3. ReadFacade 使用 `&World` 提供不可变查询
//! 4. WriteFacade 使用 `&mut World` / `Commands` 提供可变操作
//!
//! 详见 ADR-024, docs/02-domain/domains/economy_domain.md

mod command_handler;
mod facade;
mod query;

pub use command_handler::on_economy_command;
pub use facade::{EconomyReadFacade, EconomyWriteFacade};
pub use query::EconomyQueryParam;
