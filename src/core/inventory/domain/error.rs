//! 背包领域错误
//!
//! 覆盖背包与物品系统中的预期异常：容量不足、物品不存在、堆叠溢出等。
//! 错误码格式：I + 三位序号
//!
//! I001-I009: 容量/存在性错误
//! I010-I019: 转移/操作错误

use crate::shared::ids::ItemId;
use thiserror::Error;

/// 背包领域错误枚举
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InventoryError {
    /// I001: 背包容量不足
    #[error("I001: 背包容量不足: 剩余 {available} 槽位")]
    BagFull { available: u32 },

    /// I002: 物品不存在
    #[error("I002: 物品不存在: {item_id}")]
    ItemNotFound { item_id: ItemId },

    /// I003: 堆叠超过上限
    #[error("I003: 堆叠超过上限: {item_id}, 当前 {current}, 上限 {max}")]
    StackOverflow {
        item_id: ItemId,
        current: u32,
        max: u32,
    },

    /// I004: 物品不可转移
    #[error("I004: 物品不可转移: {item_id}, 原因: {reason}")]
    TransferDenied { item_id: ItemId, reason: String },

    /// I005: 容器不存在
    #[error("I005: 容器不存在: {container_id}")]
    ContainerNotFound { container_id: String },
}

/// 背包领域结果类型
pub type InventoryResult<T> = Result<T, InventoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bag_full_包含错误码() {
        let err = InventoryError::BagFull { available: 0 };
        let msg = err.to_string();
        assert!(msg.contains("I001"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn item_not_found_包含item_id() {
        let err = InventoryError::ItemNotFound {
            item_id: ItemId::new("iron_sword"),
        };
        let msg = err.to_string();
        assert!(msg.contains("iron_sword"));
    }

    #[test]
    fn inventory_result_类型可用() {
        let ok: InventoryResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: InventoryResult<i32> = Err(InventoryError::BagFull { available: 0 });
        assert!(err.is_err());
    }
}
