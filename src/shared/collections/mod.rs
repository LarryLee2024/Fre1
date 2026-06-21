//! 通用集合扩展工具
//!
//! 提供 Iterator 扩展 trait，增强集合处理能力：
//! - [`GroupByMap`]: 按键分组收集到 HashMap
//! - [`TakeWhileInclusive`] / [`TakeWhileInclusiveExt`]: 包含首个不满足谓词元素的迭代器适配器
//! - [`PartitionMap`]: 单次遍历分区为 Ok/Err 两个 Vec

mod group_by_map;
mod partition_map;
mod take_while_inclusive;

pub use group_by_map::GroupByMap;
pub use partition_map::PartitionMap;
pub use take_while_inclusive::TakeWhileInclusive;
pub use take_while_inclusive::TakeWhileInclusiveExt;

#[cfg(test)]
mod tests;
