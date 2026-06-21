//! Projections — 领域事件到 ViewModel 的投影管线
//!
//! 纯函数，将领域事件转换为 ViewModel 更新。
//! 每个投影模块处理一个领域区域（battle、character 等）。
//! Projection 无状态、确定性且可独立测试。
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

pub mod battle;
pub mod economy;

pub use battle::*;
pub use economy::*;

#[cfg(test)]
mod tests;
