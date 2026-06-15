// 装备实例：EquipmentInstance, Inventory

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 装备实例（运行时，可变）
/// 同一装备定义可以有多个实例，各自拥有不同的耐久、强化等级、附魔
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentInstance {
    /// 唯一实例 ID
    pub instance_id: u64,
    /// 指向定义 ID
    pub def_id: String,
    /// 当前耐久度
    pub durability: u32,
    /// 最大耐久度
    pub max_durability: u32,
    /// 强化等级
    #[serde(default)]
    pub enhance_level: u32,
    /// 附魔 trait
    #[serde(default)]
    pub enchantments: Vec<String>,
}

impl EquipmentInstance {
    /// 创建新实例
    pub fn new(instance_id: u64, def_id: String, max_durability: u32) -> Self {
        Self {
            instance_id,
            def_id,
            durability: max_durability,
            max_durability,
            enhance_level: 0,
            enchantments: vec![],
        }
    }
}

/// 背包组件：存储未装备的物品实例
#[derive(Component, Default, Debug, Clone)]
pub struct Inventory {
    pub items: Vec<EquipmentInstance>,
    pub capacity: u32,
}

impl Inventory {
    pub fn new(capacity: u32) -> Self {
        Self {
            items: Vec::new(),
            capacity,
        }
    }

    /// 添加物品到背包
    pub fn add(&mut self, instance: EquipmentInstance) -> bool {
        if self.items.len() >= self.capacity as usize {
            return false;
        }
        self.items.push(instance);
        true
    }

    /// 从背包移除指定实例 ID 的物品
    pub fn remove(&mut self, instance_id: u64) -> Option<EquipmentInstance> {
        if let Some(idx) = self.items.iter().position(|i| i.instance_id == instance_id) {
            Some(self.items.remove(idx))
        } else {
            None
        }
    }

    /// 查找指定实例 ID 的物品
    pub fn get(&self, instance_id: u64) -> Option<&EquipmentInstance> {
        self.items.iter().find(|i| i.instance_id == instance_id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证实例创建和背包操作结果
    // ✅ 符合领域规则：是 — 覆盖 INV-INS-1~4 装备实例不变量
    // ✅ 确定性：是 — 硬编码实例数据
    // ✅ 使用标准数据：是 — 使用标准 EquipmentInstance
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;

    #[test]
    fn 装备实例_创建() {
        let instance = EquipmentInstance::new(1, "iron_sword".into(), 100);
        assert_eq!(instance.instance_id, 1);
        assert_eq!(instance.def_id, "iron_sword");
        assert_eq!(instance.durability, 100);
        assert_eq!(instance.max_durability, 100);
        assert_eq!(instance.enhance_level, 0);
        assert!(instance.enchantments.is_empty());
    }

    #[test]
    fn 背包_添加和移除() {
        let mut inv = Inventory::new(10);
        let instance = EquipmentInstance::new(1, "iron_sword".into(), 100);
        assert!(inv.add(instance));
        assert_eq!(inv.len(), 1);

        let removed = inv.remove(1);
        assert!(removed.is_some());
        assert!(inv.is_empty());
    }

    #[test]
    fn 背包_容量限制() {
        let mut inv = Inventory::new(2);
        assert!(inv.add(EquipmentInstance::new(1, "a".into(), 100)));
        assert!(inv.add(EquipmentInstance::new(2, "b".into(), 100)));
        assert!(!inv.add(EquipmentInstance::new(3, "c".into(), 100))); // 超出容量
    }

    #[test]
    fn 背包_移除不存在的返回none() {
        let mut inv = Inventory::new(10);
        assert!(inv.remove(999).is_none());
    }

    #[test]
    fn 背包_查找() {
        let mut inv = Inventory::new(10);
        inv.add(EquipmentInstance::new(1, "iron_sword".into(), 100));
        assert!(inv.get(1).is_some());
        assert!(inv.get(999).is_none());
    }
}
