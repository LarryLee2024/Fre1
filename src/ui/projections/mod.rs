//! 领域事件到 ViewModel 的投影管道
//!
//! 将领域事件转换为 ViewModel 更新的纯函数。
//! 每个投影模块处理一个领域区域（战斗、角色等）。
//! 投影是无状态、确定性且可独立测试的。
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

pub mod battle;

pub use battle::*;

#[cfg(test)]
mod tests;
