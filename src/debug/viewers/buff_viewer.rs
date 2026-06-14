// Buff Viewer：运行时查看所有单位的 Buff 状态
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::core::buff::ActiveBuffs;
use crate::core::character::{Faction, Unit, UnitName};
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// 渲染 Buff 视图内容
pub fn render(
    ui: &mut egui::Ui,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &crate::core::character::GridPosition,
        &crate::core::attribute::Attributes,
        &crate::core::equipment::EquipmentSlots,
        &crate::core::character::TraitCollection,
        &crate::core::ability::SkillSlots,
        &crate::core::ability::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::core::character::AiBehaviorId>,
        Option<&ActiveBuffs>,
    )>,
) {
    ui.heading("Buff Viewer");

    for (entity, unit, name, _, _, _, _, _, _, _, _, buffs_opt) in units.iter() {
        let faction_label = match unit.faction {
            Faction::Player => "[友]",
            Faction::Enemy => "[敌]",
        };
        let header = format!("{}{} (e:{})", faction_label, name.0, entity.index());

        egui::CollapsingHeader::new(&header)
            .default_open(false)
            .show(ui, |ui| {
                if let Some(buffs) = buffs_opt {
                    if buffs.is_empty() {
                        ui.label("  (无 Buff)");
                    } else {
                        for buff in &buffs.instances {
                            let type_icon = if buff.is_buff { "▲" } else { "▼" };
                            let dot_label = if buff.dot_damage > 0 {
                                format!(" DoT:{}", buff.dot_damage)
                            } else {
                                String::new()
                            };
                            let hot_label = if buff.hot_heal > 0 {
                                format!(" HoT:{}", buff.hot_heal)
                            } else {
                                String::new()
                            };
                            let stun_label = if buff.tags.contains(&GameplayTag::STUN) {
                                " [晕眩]".to_string()
                            } else {
                                String::new()
                            };
                            ui.label(format!(
                                "  {} {} (id:{}) 剩余:{}回合{}{}{}",
                                type_icon,
                                buff.name,
                                buff.buff_id,
                                buff.remaining_turns,
                                dot_label,
                                hot_label,
                                stun_label,
                            ));
                        }
                    }
                } else {
                    ui.label("  (无 ActiveBuffs 组件)");
                }
            });
    }
}
