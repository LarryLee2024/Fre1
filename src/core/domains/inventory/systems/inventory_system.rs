//! Inventory Systems — 装备穿戴/卸下、物品使用、拾取等系统
//!
//! 使用 Bevy Observer 模式处理背包操作。
//! 详见 docs/02-domain/domains/inventory_domain.md §5

use bevy::prelude::*;

use crate::core::domains::inventory::components::{EquipmentSlots, Inventory, ItemInstance};
use crate::core::domains::inventory::events::{EquipmentChanged, ItemAcquired, ItemUsed};

/// 拾取物品系统。
///
/// 处理 ItemAcquired 事件：将物品加入背包。
pub(crate) fn on_item_acquired(
    trigger: On<ItemAcquired>,
    mut query: Query<&mut Inventory>,
    _commands: Commands,
) {
    let ev = trigger.event();
    let Ok(mut inventory) = query.get_mut(ev.entity) else {
        tracing::warn!(
            event = "inventory.item_acquired.missing_component",
            entity = ?ev.entity,
            template = %ev.item_template_id,
            "ItemAcquired: 实体 {:?} 没有 Inventory 组件",
            ev.entity
        );
        return;
    };

    let item = ItemInstance::with_quantity(&ev.item_template_id, ev.quantity);
    // 使用默认重量 1.0 磅 — 实际重量应由 ItemDef 提供
    let added = inventory.add_item(item, 1.0);

    if added > 0 {
        tracing::trace!(
            event = "inventory.item_acquired.added",
            entity = ?ev.entity,
            template = %ev.item_template_id,
            qty = added,
            "物品获取成功：实体={:?}, 模板={}, 数量={}",
            ev.entity, ev.item_template_id, added
        );
    } else {
        tracing::warn!(
            event = "inventory.item_acquired.failed",
            entity = ?ev.entity,
            template = %ev.item_template_id,
            qty_requested = ev.quantity,
            "物品获取失败：实体={:?}, 模板={}, 请求数量={}",
            ev.entity, ev.item_template_id, ev.quantity
        );
    }
}

/// 装备穿戴系统。
///
/// 将物品从背包移动到装备槽位。
/// 注意：装备条件检查应在触发此事件前完成（通过 Condition 领域）。
pub(crate) fn on_equip_item(
    trigger: On<EquipmentChanged>,
    mut query: Query<(&mut Inventory, &mut EquipmentSlots)>,
    _commands: Commands,
) {
    let ev = trigger.event();
    let Ok((mut inventory, mut equipment)) = query.get_mut(ev.entity) else {
        tracing::warn!(
            event = "inventory.equipment_changed.missing_components",
            entity = ?ev.entity,
            slot = ?ev.slot,
            "EquipmentChanged: 实体 {:?} 没有 Inventory/EquipmentSlots 组件",
            ev.entity
        );
        return;
    };

    // 如果是穿戴（new_item 有值）
    if let Some(ref new_template_id) = ev.new_item_template_id {
        // 从背包查找并移除物品
        let removed_qty = inventory.remove_item(new_template_id, 1, 1.0);
        if removed_qty > 0 {
            let item = ItemInstance::new(new_template_id);
            let old = equipment.equip(ev.slot, item);

            // 如果旧装备存在，放回背包
            if let Some(old_item) = old {
                let old_template = old_item.template_id.clone();
                inventory.add_item(old_item, 1.0);
                tracing::trace!(
                    event = "inventory.equipment_changed.replaced",
                    entity = ?ev.entity,
                    slot = ?ev.slot,
                    new = %new_template_id,
                    replaced = %old_template,
                    "装备更换：实体={:?}, 槽位={:?}, 新={}, 旧={}",
                    ev.entity, ev.slot, new_template_id, old_template
                );
            } else {
                tracing::trace!(
                    event = "inventory.equipment_changed.equipped",
                    entity = ?ev.entity,
                    slot = ?ev.slot,
                    item = %new_template_id,
                    "装备穿戴：实体={:?}, 槽位={:?}, 物品={}",
                    ev.entity, ev.slot, new_template_id
                );
            }
        }
    }

    // 如果是卸下（new_item 为空，old_item 有值）
    if ev.new_item_template_id.is_none()
        && let Some(ref old_template_id) = ev.old_item_template_id
    {
        let old_item = equipment.unequip(ev.slot);
        if let Some(item) = old_item {
            inventory.add_item(item, 1.0);
            tracing::trace!(
                event = "inventory.equipment_changed.unequipped",
                entity = ?ev.entity,
                slot = ?ev.slot,
                item = %old_template_id,
                "装备卸下：实体={:?}, 槽位={:?}, 物品={}",
                ev.entity, ev.slot, old_template_id
            );
        }
    }
}

/// 物品使用系统。
///
/// 处理 ItemUsed 事件：消耗背包中的物品数量。
pub(crate) fn on_item_used(trigger: On<ItemUsed>, mut query: Query<&mut Inventory>) {
    let ev = trigger.event();
    let Ok(mut inventory) = query.get_mut(ev.entity) else {
        tracing::warn!(
            event = "inventory.item_used.missing_component",
            entity = ?ev.entity,
            "ItemUsed: 实体 {:?} 没有 Inventory 组件",
            ev.entity
        );
        return;
    };

    // 检查是否拥有足够数量
    if !inventory.has_item(&ev.item_template_id, ev.quantity_consumed) {
        tracing::warn!(
            event = "inventory.item_used.insufficient_quantity",
            entity = ?ev.entity,
            template = %ev.item_template_id,
            "ItemUsed: 实体 {:?} 的物品 {} 数量不足",
            ev.entity, ev.item_template_id
        );
        return;
    }

    let removed = inventory.remove_item(&ev.item_template_id, ev.quantity_consumed, 1.0);
    if removed > 0 {
        tracing::trace!(
            event = "inventory.item_used.consumed",
            entity = ?ev.entity,
            template = %ev.item_template_id,
            consumed = removed,
            "物品使用：实体={:?}, 模板={}, 消耗={}",
            ev.entity, ev.item_template_id, removed
        );
    }
}
