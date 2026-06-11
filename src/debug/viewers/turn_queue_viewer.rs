// Turn Queue Viewer：回合队列查看器
// 遵循铁律：关键系统必须拥有可视化观察窗口

use crate::character::{Faction, Unit, UnitName};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::turn::TurnOrder;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;

/// 回合队列查看器（由 mod.rs 的条件渲染系统调用）
pub fn turn_queue_viewer_system_inner(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    turn_order: Res<TurnOrder>,
    units: Query<(&UnitName, &Unit, &Attributes)>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    bevy_inspector_egui::egui::Window::new("Turn Queue")
        .default_pos([10.0, 600.0])
        .default_size([280.0, 300.0])
        .show(ctx, |ui| {
            ui.label(format!("Round {}", turn_order.turn_number));
            ui.separator();

            if turn_order.queue.is_empty() {
                ui.label("等待回合开始");
                return;
            }

            for (i, entity) in turn_order.queue.iter().enumerate() {
                let is_current = i == turn_order.current_index;
                let Ok((name, unit, attrs)) = units.get(*entity) else {
                    ui.label(format!(
                        "  {} Entity({:?})",
                        if is_current { "▶" } else { " " },
                        entity
                    ));
                    continue;
                };

                let faction_label = match unit.faction {
                    Faction::Player => "[友]",
                    Faction::Enemy => "[敌]",
                };
                let initiative = attrs.get(AttributeKind::Initiative);
                let marker = if is_current { "▶" } else { " " };

                let text = format!(
                    "{} {} {} SPD={:.0} e:{}",
                    marker,
                    faction_label,
                    name.0,
                    initiative,
                    entity.index()
                );
                if is_current {
                    ui.colored_label(
                        bevy_inspector_egui::egui::Color32::from_rgb(255, 255, 100),
                        text,
                    );
                } else {
                    ui.label(text);
                }
            }
        });
}
