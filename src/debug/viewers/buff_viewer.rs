// Buff Viewer：运行时查看所有单位的 Buff 状态
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::buff::ActiveBuffs;
use crate::character::{Faction, Unit, UnitName};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// Buff Viewer 调试面板
pub fn buff_viewer_system(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    units: Query<(Entity, &Unit, &UnitName, &ActiveBuffs)>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Buff Viewer")
        .default_pos([10.0, 200.0])
        .default_size([350.0, 400.0])
        .show(ctx, |ui| {
            for (entity, unit, name, buffs) in &units {
                let faction_label = match unit.faction {
                    Faction::Player => "[友]",
                    Faction::Enemy => "[敌]",
                };
                let header = format!("{}{} (e:{})", faction_label, name.0, entity.index());

                egui::CollapsingHeader::new(&header)
                    .default_open(buffs.len() > 0)
                    .show(ui, |ui| {
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
                                let stun_label =
                                    if buff.tags.contains(&crate::core::tag::GameplayTag::STUN) {
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
                    });
            }
        });
}
