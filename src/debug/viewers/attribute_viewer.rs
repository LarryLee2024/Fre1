// Attribute Modifier Viewer：属性修饰符来源分解面板
// 遵循铁律：关键系统必须拥有可视化观察窗口
// 数据源：Attributes.modifiers + ModifierSource 区间判断

use crate::character::{Faction, TraitCollection, Unit, UnitName};
use crate::core::attribute::{
    AttributeKind, AttributeModifierInstance, Attributes, ModifierOp, ModifierSource,
};
use crate::equipment::EquipmentSlots;
use bevy::prelude::*;

/// 渲染属性修饰符面板内容（由 mod.rs 的条件渲染系统调用）
pub fn render_attribute_panel(
    ui: &mut bevy_inspector_egui::egui::Ui,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &Attributes,
        &EquipmentSlots,
        &TraitCollection,
    )>,
) {
    for (entity, unit, name, attrs, slots, trait_collection) in units {
        let faction_label = match unit.faction {
            Faction::Player => "[友]",
            Faction::Enemy => "[敌]",
        };
        let header = format!("{}{} (e:{})", faction_label, name.0, entity.index());

        bevy_inspector_egui::egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                render_attributes(ui, attrs, slots, trait_collection);
            });
    }
}

fn render_attributes(
    ui: &mut bevy_inspector_egui::egui::Ui,
    attrs: &Attributes,
    slots: &EquipmentSlots,
    trait_collection: &TraitCollection,
) {
    // 收集所有有修饰符的属性类型
    let mut kinds_with_mods: Vec<AttributeKind> = attrs.modifiers.iter().map(|m| m.kind).collect();
    kinds_with_mods.sort_by_key(|k| format!("{:?}", k));
    kinds_with_mods.dedup();

    // 生命资源
    ui.label(format!(
        "HP: {:.0} / {:.0}",
        attrs.current_hp,
        attrs.get(AttributeKind::MaxHp)
    ));
    ui.label(format!(
        "MP: {:.0} / {:.0}",
        attrs.current_mp,
        attrs.get(AttributeKind::MaxMp)
    ));

    ui.separator();

    // 核心属性（8维）
    let core_kinds = [
        AttributeKind::Might,
        AttributeKind::Dexterity,
        AttributeKind::Agility,
        AttributeKind::Vitality,
        AttributeKind::Intelligence,
        AttributeKind::Willpower,
        AttributeKind::Presence,
        AttributeKind::Luck,
    ];

    for kind in &core_kinds {
        let label = kind.label();
        let base = attrs.core_base(*kind);
        let final_val = attrs.core(*kind);

        if kinds_with_mods.contains(kind) {
            bevy_inspector_egui::egui::CollapsingHeader::new(format!(
                "{} = {:.0}",
                label, final_val
            ))
            .default_open(false)
            .show(ui, |ui| {
                ui.label(format!("  基础: {:.0}", base));
                render_modifiers_for_kind(ui, &attrs.modifiers, *kind, slots, trait_collection);
            });
        } else {
            ui.label(format!("{} = {:.0}", label, final_val));
        }
    }
}

fn render_modifiers_for_kind(
    ui: &mut bevy_inspector_egui::egui::Ui,
    modifiers: &[AttributeModifierInstance],
    kind: AttributeKind,
    slots: &EquipmentSlots,
    trait_collection: &TraitCollection,
) {
    let kind_mods: Vec<_> = modifiers.iter().filter(|m| m.kind == kind).collect();
    if kind_mods.is_empty() {
        return;
    }

    for m in kind_mods {
        let source_label = classify_source(m.source, slots, trait_collection);
        let op_label = match m.op {
            ModifierOp::Add => format!("+{:.0}", m.value),
            ModifierOp::Multiply => format!("x{:.2}", m.value),
        };

        let color = if m.value < 0.0 {
            bevy_inspector_egui::egui::Color32::from_rgb(255, 100, 100)
        } else {
            bevy_inspector_egui::egui::Color32::from_rgb(100, 255, 100)
        };

        ui.colored_label(color, format!("  {} ({})", op_label, source_label));
    }
}

fn classify_source(
    source: ModifierSource,
    slots: &EquipmentSlots,
    trait_collection: &TraitCollection,
) -> String {
    if source.is_trait() {
        let trait_index = u64::MAX - source.0;
        // 尝试匹配 TraitCollection 中的条目
        if let Some(entry) = trait_collection.entries.iter().find(|e| {
            trait_collection
                .entries
                .iter()
                .position(|x| x.trait_id == e.trait_id)
                == Some(trait_index as usize)
        }) {
            format!("Trait({})", entry.trait_id)
        } else {
            format!("Trait(#{})", trait_index)
        }
    } else if source.is_equipment() {
        let equip_index = (u64::MAX - 1000) - source.0;
        let equipped: Vec<_> = slots.equipped_slots();
        if let Some((_, _, def_id)) = equipped.iter().find(|(s, _, _)| {
            let slot_index = match s {
                crate::equipment::EquipmentSlot::MainHand => 0,
                crate::equipment::EquipmentSlot::OffHand => 1,
                crate::equipment::EquipmentSlot::Head => 2,
                crate::equipment::EquipmentSlot::Body => 3,
                crate::equipment::EquipmentSlot::Legs => 4,
                crate::equipment::EquipmentSlot::Feet => 5,
                crate::equipment::EquipmentSlot::Accessory1 => 6,
                crate::equipment::EquipmentSlot::Accessory2 => 7,
            };
            slot_index as u64 == equip_index
        }) {
            format!("装备({})", def_id)
        } else {
            format!("装备(#{})", equip_index)
        }
    } else if source.is_buff() {
        format!("Buff(#{})", source.0)
    } else {
        format!("未知({})", source.0)
    }
}
