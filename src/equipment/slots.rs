// 装备槽位：EquipmentSlots 组件

use super::definition::EquipmentSlot;
use bevy::prelude::*;
use std::collections::HashMap;

/// 装备槽组件：记录每个槽位装备了哪个实例
#[derive(Component, Default, Debug, Clone)]
pub struct EquipmentSlots {
    /// 槽位 → 装备实例 ID
    pub slots: HashMap<EquipmentSlot, u64>,
    /// 下一个实例 ID（自增）
    pub next_instance_id: u64,
}

impl EquipmentSlots {
    /// 获取指定槽位的装备实例 ID
    pub fn get(&self, slot: EquipmentSlot) -> Option<u64> {
        self.slots.get(&slot).copied()
    }

    /// 是否已装备指定槽位
    pub fn is_equipped(&self, slot: EquipmentSlot) -> bool {
        self.slots.contains_key(&slot)
    }

    /// 生成下一个唯一实例 ID
    pub fn next_instance_id(&mut self) -> u64 {
        self.next_instance_id += 1;
        self.next_instance_id
    }

    /// 装备到指定槽位，返回被替换的旧实例 ID（如果有）
    pub fn equip(&mut self, slot: EquipmentSlot, instance_id: u64) -> Option<u64> {
        self.slots.insert(slot, instance_id)
    }

    /// 卸下指定槽位，返回被卸下的实例 ID
    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<u64> {
        self.slots.remove(&slot)
    }

    /// 获取所有已装备的槽位
    pub fn equipped_slots(&self) -> Vec<(EquipmentSlot, u64)> {
        self.slots.iter().map(|(s, id)| (*s, *id)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 装备槽_装备和卸下() {
        let mut slots = EquipmentSlots::default();
        assert!(!slots.is_equipped(EquipmentSlot::MainHand));

        slots.equip(EquipmentSlot::MainHand, 1);
        assert!(slots.is_equipped(EquipmentSlot::MainHand));
        assert_eq!(slots.get(EquipmentSlot::MainHand), Some(1));

        let removed = slots.unequip(EquipmentSlot::MainHand);
        assert_eq!(removed, Some(1));
        assert!(!slots.is_equipped(EquipmentSlot::MainHand));
    }

    #[test]
    fn 装备槽_替换旧装备() {
        let mut slots = EquipmentSlots::default();
        slots.equip(EquipmentSlot::MainHand, 1);
        let old = slots.equip(EquipmentSlot::MainHand, 2);
        assert_eq!(old, Some(1));
        assert_eq!(slots.get(EquipmentSlot::MainHand), Some(2));
    }

    #[test]
    fn 装备槽_卸下空槽位() {
        let mut slots = EquipmentSlots::default();
        let result = slots.unequip(EquipmentSlot::MainHand);
        assert!(result.is_none());
    }

    #[test]
    fn 装备槽_实例id自增() {
        let mut slots = EquipmentSlots::default();
        let id1 = slots.next_instance_id();
        let id2 = slots.next_instance_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn 装备槽_多槽位() {
        let mut slots = EquipmentSlots::default();
        slots.equip(EquipmentSlot::MainHand, 1);
        slots.equip(EquipmentSlot::Body, 2);
        slots.equip(EquipmentSlot::Accessory1, 3);
        assert_eq!(slots.equipped_slots().len(), 3);
    }
}
