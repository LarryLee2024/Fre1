// Equipment Viewer：运行时查看所有单位的装备和背包状态
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::character::{Faction, TraitCollection, Unit, UnitName};
use crate::equipment::{EquipmentRegistry, EquipmentSlots, Inventory};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// Equipment Viewer 调试面板
pub fn equipment_viewer_system(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    units: Query<(
        Entity,
        &Unit,
        &UnitName,
        &EquipmentSlots,
        &Inventory,
        &TraitCollection,
    )>,
    equipment_registry: Res<EquipmentRegistry>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Equipment Viewer")
        .default_pos([10.0, 400.0])
        .default_size([400.0, 450.0])
        .show(ctx, |ui| {
            for (entity, unit, name, slots, inventory, trait_collection) in &units {
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
                                if let Some(def) = equipment_registry.get(&def_id) {
                                    let rarity_label = def.rarity.label();
                                    let tags_str = def
                                        .tags
                                        .iter()
                                        .map(|t| format!("{:?}", t))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    ui.label(format!(
                                        "  {}: {} [{}] {}",
                                        slot_label, def.name, rarity_label, tags_str,
                                    ));
                                    // 修饰符
                                    if !def.modifiers.is_empty() {
                                        for mod_def in &def.modifiers {
                                            ui.label(format!(
                                                "    {} {:?} {}",
                                                format!("{:?}", mod_def.kind),
                                                mod_def.op,
                                                mod_def.value,
                                            ));
                                        }
                                    }
                                    // Trait
                                    if !def.traits.is_empty() {
                                        let traits_str = def.traits.join(", ");
                                        ui.label(format!("    Traits: {}", traits_str));
                                    }
                                } else {
                                    ui.label(format!(
                                        "  {}: ??? (def_id={}, inst={})",
                                        slot_label, def_id, instance_id,
                                    ));
                                }
                            }
                        }

                        // 背包
                        ui.label("── 背包 ──");
                        if inventory.items.is_empty() {
                            ui.label("  (空)");
                        } else {
                            for instance in &inventory.items {
                                if let Some(def) = equipment_registry.get(&instance.def_id) {
                                    let rarity_label = def.rarity.label();
                                    ui.label(format!(
                                        "  {} [{}] 耐久:{} id:{}",
                                        def.name,
                                        rarity_label,
                                        instance.durability,
                                        instance.instance_id,
                                    ));
                                } else {
                                    ui.label(format!(
                                        "  ??? (def_id={}, id:{})",
                                        instance.def_id, instance.instance_id,
                                    ));
                                }
                            }
                        }

                        // Trait 来源
                        ui.label("── Trait 来源 ──");
                        if trait_collection.entries.is_empty() {
                            ui.label("  (无)");
                        } else {
                            for entry in &trait_collection.entries {
                                let source_label = match &entry.source {
                                    crate::character::TraitSource::Intrinsic => "内在".to_string(),
                                    crate::character::TraitSource::Equipment { slot } => {
                                        format!("装备({})", slot.label())
                                    }
                                };
                                ui.label(format!("  {} [{}]", entry.trait_id, source_label));
                            }
                        }
                    });
            }
        });
}
