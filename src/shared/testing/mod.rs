//! 测试构建工具
//!
//! 提供对领域无关的测试辅助函数和 mock 构造器。
//! 领域内聚测试参见各领域内的 `tests/` 模块。

pub mod assertions;
pub mod deterministic;
pub mod fixtures;

pub use assertions::*;
pub use deterministic::*;
pub use fixtures::*;
