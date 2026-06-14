// Damage & Attribute Viewer：伤害分解与属性修饰符来源分解面板
// 遵循铁律：关键系统必须拥有可视化观察窗口

use crate::battle::{BattleEntry, BattleRecord, DamageBreakdown};
use crate::character::{Faction, TraitCollection, Unit, UnitName};
use crate::core::attribute::{
    AttributeKind, AttributeModifierInstance, Attributes, ModifierOp, ModifierSource,
};
use crate::equipment::EquipmentSlots;
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// 伤害条目（纯数据，Logic/Presentation 分离）
pub struct DamageEntry {
    pub target: Entity,
    pub target_name: String,
    pub attacker_name: String,
    pub amount: i32,
    pub is_skill: bool,
    pub breakdown: Option<DamageBreakdown>,
}

/// 提取最近 N 条伤害记录
pub fn filter_damage_entries(record: &BattleRecord, limit: usize) -> Vec<DamageEntry> {
    record
        .entries
        .iter()
        .rev()
        .filter_map(|e| {
            if let BattleEntry::DamageApplied {
                target,
                target_name,
                attacker_name,
                amount,
                is_skill,
                breakdown,
                ..
            } = e
            {
                Some(DamageEntry {
                    target: *target,
                    target_name: target_name.clone(),
                    attacker_name: attacker_name.clone(),
                    amount: *amount,
                    is_skill: *is_skill,
                    breakdown: breakdown.clone(),
                })
            } else {
                None
            }
        })
        .take(limit)
        .collect()
}

/// 提取有修饰符的属性类型
pub fn group_modifiers_by_kind(attrs: &Attributes) -> Vec<AttributeKind> {
    let mut kinds: Vec<AttributeKind> = attrs.modifiers.iter().map(|m| m.kind).collect();
    kinds.sort_by_key(|k| format!("{:?}", k));
    kinds.dedup();
    kinds
}

/// 渲染 Damage & Attribute 视图内容
pub fn render(
    ui: &mut egui::Ui,
    tab: &mut u32,
    battle_record: &BattleRecord,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &crate::character::GridPosition,
        &Attributes,
        &EquipmentSlots,
        &TraitCollection,
        &crate::skill::SkillSlots,
        &crate::skill::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::character::AiBehaviorId>,
        Option<&crate::buff::ActiveBuffs>,
    )>,
    _unit_names: &Query<&UnitName>,
) {
    ui.heading("Damage & Attribute Debugger");

    // Tab 切换
    ui.horizontal(|ui| {
        let damage_selected = ui.selectable_label(*tab == 0, "Damage Breakdown").clicked();
        let attr_selected = ui
            .selectable_label(*tab == 1, "Attribute Modifier")
            .clicked();
        if damage_selected {
            *tab = 0;
        }
        if attr_selected {
            *tab = 1;
        }
    });
    ui.separator();

    if *tab == 0 {
        render_damage_panel(ui, battle_record);
    } else {
        render_attribute_panel(ui, units);
    }
}

fn render_damage_panel(ui: &mut egui::Ui, battle_record: &BattleRecord) {
    let damage_entries = filter_damage_entries(battle_record, 20);

    if damage_entries.is_empty() {
        ui.label("暂无伤害记录");
        return;
    }

    for (idx, entry) in damage_entries.iter().enumerate() {
        let skill_label = if entry.is_skill { " [技能]" } else { "" };
        let header = format!(
            "#{} {} → {}  伤害:{}{}",
            idx + 1,
            entry.attacker_name,
            entry.target_name,
            entry.amount,
            skill_label
        );

        egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                if let Some(bd) = &entry.breakdown {
                    render_breakdown(ui, bd);
                } else {
                    ui.label("  (无分解数据)");
                    ui.label(format!("  最终伤害: {}", entry.amount));
                }
            });
    }
}

fn render_breakdown(ui: &mut egui::Ui, bd: &DamageBreakdown) {
    ui.label(format!("  原始效果值: {}", bd.base_amount));

    if !bd.modifiers.is_empty() {
        ui.label("  修饰符:");
        for m in &bd.modifiers {
            let sign = if m.after >= m.before { "+" } else { "" };
            let diff = m.after - m.before;
            ui.label(format!("    {}{} ({})", sign, diff, m.rule_name));
        }
    }

    ui.label(format!("  修饰后伤害: {}", bd.modified_amount));

    if bd.actual_damage != bd.modified_amount {
        let diff = bd.actual_damage - bd.modified_amount;
        let sign = if diff >= 0 { "+" } else { "" };
        ui.label(format!(
            "  实际扣血: {} ({}{})",
            bd.actual_damage, sign, diff
        ));
    } else {
        ui.label(format!("  实际扣血: {}", bd.actual_damage));
    }
}

fn render_attribute_panel(
    ui: &mut egui::Ui,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &crate::character::GridPosition,
        &Attributes,
        &EquipmentSlots,
        &TraitCollection,
        &crate::skill::SkillSlots,
        &crate::skill::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::character::AiBehaviorId>,
        Option<&crate::buff::ActiveBuffs>,
    )>,
) {
    for (entity, unit, name, _, attrs, slots, trait_collection, ..) in units.iter() {
        let faction_label = match unit.faction {
            Faction::Player => "[友]",
            Faction::Enemy => "[敌]",
        };
        let header = format!("{}{} (e:{})", faction_label, name.0, entity.index());

        egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                render_attributes(ui, attrs, slots, trait_collection);
            });
    }
}

fn render_attributes(
    ui: &mut egui::Ui,
    attrs: &Attributes,
    slots: &EquipmentSlots,
    trait_collection: &TraitCollection,
) {
    let kinds_with_mods = group_modifiers_by_kind(attrs);

    ui.label(format!(
        "HP: {:.0} / {:.0}",
        attrs.get(AttributeKind::Hp),
        attrs.get(AttributeKind::MaxHp)
    ));
    ui.label(format!(
        "MP: {:.0} / {:.0}",
        attrs.get(AttributeKind::Mp),
        attrs.get(AttributeKind::MaxMp)
    ));

    ui.separator();

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
            egui::CollapsingHeader::new(format!("{} = {:.0}", label, final_val))
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
    ui: &mut egui::Ui,
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
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(100, 255, 100)
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
