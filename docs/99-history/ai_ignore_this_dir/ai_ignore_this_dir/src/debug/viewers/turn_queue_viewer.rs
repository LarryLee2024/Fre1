// Turn Queue Viewer：回合队列查看器
// 遵循铁律：关键系统必须拥有可视化观察窗口

use crate::core::attribute::Attributes;
use crate::core::character::{Faction, Unit, UnitName};
use crate::core::turn::TurnOrder;
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// 渲染 Turn Queue 视图内容
pub fn render(
    ui: &mut egui::Ui,
    turn_order: &TurnOrder,
    units: &Query<(
        Entity,
        &Unit,
        &UnitName,
        &crate::core::character::GridPosition,
        &Attributes,
        &crate::core::equipment::EquipmentSlots,
        &crate::core::character::TraitCollection,
        &crate::core::ability::SkillSlots,
        &crate::core::ability::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::core::character::AiBehaviorId>,
        Option<&crate::core::buff::ActiveBuffs>,
    )>,
) {
    ui.heading("Turn Queue");
    ui.label(format!("Round {}", turn_order.turn_number));
    ui.separator();

    if turn_order.queue.is_empty() {
        ui.label("等待回合开始");
        return;
    }

    for (i, entity) in turn_order.queue.iter().enumerate() {
        let is_current = i == turn_order.current_index;
        let Ok((_, unit, name, _, attrs, ..)) = units.get(*entity) else {
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
        let initiative = attrs.get("initiative");
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
}
