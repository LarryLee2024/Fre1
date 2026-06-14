/// 背包领域错误
///
/// ADR-004 §决策: 分领域错误枚举
/// 覆盖背包与物品系统中的预期异常：容量不足、物品不存在、堆叠溢出等。
use crate::shared::ids::ItemId;
use thiserror::Error;

/// 背包领域错误
///
/// 错误码格式：I + 三位序号
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_error_背包已满() {
        let err = InventoryError::BagFull { available: 0 };
        assert!(err.to_string().contains("I001"));
        assert!(err.to_string().contains("0"));
    }

    #[test]
    fn inventory_error_物品不存在() {
        let err = InventoryError::ItemNotFound {
            item_id: ItemId::new("iron_sword"),
        };
        assert!(err.to_string().contains("iron_sword"));
    }
}
