// AI Viewer：运行时查看 AI 决策状态
// 遵循铁律：复杂系统必须有可视化调试工具

use crate::battle::CombatIntent;
use crate::character::{AiBehaviorId, Faction, GridPosition, Unit, UnitName};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
use crate::skill::{SkillCooldowns, SkillSlots};
use crate::turn::TurnOrder;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// AI Viewer 调试面板
pub fn ai_viewer_system(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    turn_order: Res<TurnOrder>,
    combat_intent: Res<CombatIntent>,
    units: Query<(
        Entity,
        &Unit,
        &UnitName,
        &GridPosition,
        &Attributes,
        &SkillSlots,
        &SkillCooldowns,
        &GameplayTags,
        Option<&AiBehaviorId>,
    )>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("AI Viewer")
        .default_pos([370.0, 200.0])
        .default_size([350.0, 400.0])
        .show(ctx, |ui| {
            // 当前行动单位
            if let Some(current) = turn_order.current_unit() {
                ui.label(format!("当前行动单位: e:{}", current.index()));
            } else {
                ui.label("当前行动单位: (无)");
            }

            ui.separator();

            // CombatIntent 状态
            ui.heading("CombatIntent");
            if let Some(source) = combat_intent.source_entity {
                ui.label(format!("攻击者: e:{}", source.index()));
            } else {
                ui.label("攻击者: (无)");
            }
            if let Some(coord) = combat_intent.target_coord {
                ui.label(format!("目标坐标: ({}, {})", coord.x, coord.y));
            } else {
                ui.label("目标坐标: (无)");
            }
            if let Some(ref skill) = combat_intent.skill_id {
                ui.label(format!("技能: {}", skill));
            } else {
                ui.label("技能: (无)");
            }

            ui.separator();

            // 敌方单位详情
            ui.heading("敌方单位");
            for (entity, unit, name, gp, attrs, skills, cooldowns, tags, ai_id) in &units {
                if unit.faction != Faction::Enemy {
                    continue;
                }

                let ai_label = ai_id
                    .map(|id| format!(" [AI:{}]", id.0))
                    .unwrap_or_default();
                let header = format!("{} (e:{}){}", name.0, entity.index(), ai_label);

                egui::CollapsingHeader::new(&header)
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(format!("  位置: ({}, {})", gp.coord.x, gp.coord.y));
                        ui.label(format!(
                            "  HP: {:.0}/{:.0}  MP: {:.0}/{:.0}",
                            attrs.get(AttributeKind::Hp),
                            attrs.get(AttributeKind::MaxHp),
                            attrs.get(AttributeKind::Mp),
                            attrs.get(AttributeKind::MaxMp),
                        ));
                        ui.label(format!(
                            "  ATK:{:.0} DEF:{:.0} MOV:{:.0} RNG:{:.0}",
                            attrs.get(AttributeKind::Attack),
                            attrs.get(AttributeKind::Defense),
                            attrs.get(AttributeKind::MoveRange),
                            attrs.get(AttributeKind::AttackRange),
                        ));
                        ui.label(format!(
                            "  行动: {}",
                            if unit.acted { "已行动" } else { "待命" }
                        ));

                        // 技能冷却
                        let on_cd: Vec<String> = skills
                            .skill_ids
                            .iter()
                            .filter_map(|id| {
                                let cd = cooldowns.get(id);
                                if cd > 0 {
                                    Some(format!("{}({}回合)", id, cd))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if on_cd.is_empty() {
                            ui.label("  冷却: (无)");
                        } else {
                            ui.label(format!("  冷却: {}", on_cd.join(", ")));
                        }

                        // 标签
                        let tag_names: Vec<String> = tags
                            .active_tags()
                            .iter()
                            .map(|t| t.label().to_string())
                            .collect();
                        if tag_names.is_empty() {
                            ui.label("  标签: (无)");
                        } else {
                            ui.label(format!("  标签: {}", tag_names.join(", ")));
                        }
                    });
            }
        });
}
