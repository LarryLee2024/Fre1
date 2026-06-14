// Equipment Viewer：运行时查看所有单位的装备和背包状态
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::core::character::{Faction, TraitCollection, Unit, UnitName};
use crate::core::equipment::{EquipmentRegistry, EquipmentSlots, Inventory};
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// 渲染 Equipment 视图内容
pub fn render(
    ui: &mut egui::Ui,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &crate::core::character::GridPosition,
        &crate::core::attribute::Attributes,
        &EquipmentSlots,
        &TraitCollection,
        &crate::core::skill::SkillSlots,
        &crate::core::skill::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::core::character::AiBehaviorId>,
        Option<&crate::core::buff::ActiveBuffs>,
    )>,
) {
    ui.heading("Equipment Viewer");

    for (entity, unit, name, _, _, slots, trait_collection, _, _, _, _, _) in units.iter() {
        let faction_label = match unit.faction {
            Faction::Player => "[友]",
            Faction::Enemy => "[敌]",
        };
        let header = format!("{}{} (e:{})", faction_label, name.0, entity.index());

        egui::CollapsingHeader::new(&header)
            .default_open(!slots.slots.is_empty())
            .show(ui, |ui| {
                // 已装备槽位
                ui.label("── 装备槽 ──");
                if slots.slots.is_empty() {
                    ui.label("  (空)");
                } else {
                    let mut equipped: Vec<_> = slots.equipped_slots();
                    equipped.sort_by_key(|(slot, _, _)| format!("{:?}", slot));
                    for (slot, instance_id, def_id) in equipped {
                        let slot_label = slot.label();
                        ui.label(format!(
                            "  {}: def={} inst={}",
                            slot_label, def_id, instance_id,
                        ));
                    }
                }

                // 背包
                ui.label("── 背包 ──");
                if slots.slots.is_empty() {
                    ui.label("  (空)");
                } else {
                    ui.label("  (需要 Inventory 组件)");
                }

                // Trait 来源
                ui.label("── Trait 来源 ──");
                if trait_collection.entries.is_empty() {
                    ui.label("  (无)");
                } else {
                    for entry in &trait_collection.entries {
                        let source_label = match &entry.source {
                            crate::core::character::TraitSource::Intrinsic => "内在".to_string(),
                            crate::core::character::TraitSource::Equipment { slot } => {
                                format!("装备({})", slot.label())
                            }
                        };
                        ui.label(format!("  {} [{}]", entry.trait_id, source_label));
                    }
                }
            });
    }
}
