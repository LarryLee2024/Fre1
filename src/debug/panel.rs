// 调试面板渲染：统一调试面板 + World Inspector
// 在 EguiPrimaryContextPass Schedule 中运行

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContext, PrimaryEguiContext};
use bevy_inspector_egui::egui;

use super::overlay::{self, DebugOverlay};
use super::state::{DebugPanelState, DebugView, WorldInspectorState};
use super::stepping_control::{self, DebugSteppingState};
use super::viewers;

/// 统一调试面板系统
#[allow(clippy::too_many_arguments)]
pub fn unified_debug_panel(
    mut state: ResMut<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut stepping: ResMut<bevy::ecs::schedule::Stepping>,
    mut stepping_state: ResMut<DebugSteppingState>,
    mut debug_overlay: ResMut<DebugOverlay>,
    battle_record: Res<crate::core::battle::BattleRecord>,
    turn_order: Res<crate::core::turn::TurnOrder>,
    combat_intent: Res<crate::core::battle::CombatIntent>,
    terrain_grid: Res<crate::core::map::runtime::TerrainGrid>,
    terrain_registry: Res<crate::core::map::TerrainRegistry>,
    occupancy: Res<crate::core::map::runtime::OccupancyGrid>,
    mut grid_viewer_state: ResMut<viewers::GridViewerState>,
    mut settings: ResMut<crate::ui::settings::GameSettings>,
    tag_registry: Res<crate::core::tag_def::TagRegistry>,
    units: Query<(
        Entity,
        &crate::core::character::Unit,
        &crate::core::character::UnitName,
        &crate::core::character::GridPosition,
        &crate::core::attribute::Attributes,
        &crate::core::equipment::EquipmentSlots,
        &crate::core::character::TraitCollection,
        &crate::core::ability::SkillSlots,
        &crate::core::ability::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::core::character::AiBehaviorId>,
        Option<&crate::core::buff::ActiveBuffs>,
    )>,
    unit_names: Query<&crate::core::character::UnitName>,
) {
    if !state.show_panel {
        return;
    }

    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Debug Panel")
        .default_pos([10.0, 10.0])
        .default_size([520.0, 600.0])
        .resizable(true)
        .default_open(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                render_navigation(ui, &mut state);
                ui.separator();
                render_content(
                    ui,
                    &mut state,
                    &mut stepping,
                    &mut stepping_state,
                    &mut debug_overlay,
                    &battle_record,
                    &turn_order,
                    &combat_intent,
                    &terrain_grid,
                    &terrain_registry,
                    &occupancy,
                    &mut grid_viewer_state,
                    &mut settings,
                    &tag_registry,
                    &units,
                    &unit_names,
                );
            });

            ui.separator();
            ui.horizontal(|ui| {
                let stepping_label = if stepping.is_enabled() {
                    "Stepping: PAUSED"
                } else {
                    "Stepping: RUNNING"
                };
                ui.label(stepping_label);
                ui.separator();
                ui.label("F1: Panel | F2-F5: View | F6: Pause | F7: Step");
            });
        });
}

/// 渲染侧边栏导航
fn render_navigation(ui: &mut egui::Ui, state: &mut DebugPanelState) {
    ui.vertical(|ui| {
        ui.set_min_width(80.0);
        for &(view, label, shortcut) in DebugView::all() {
            let is_selected = state.active_view == view;
            let clicked = ui
                .horizontal(|ui| {
                    let response = ui.selectable_label(is_selected, label);
                    if !shortcut.is_empty() {
                        ui.label(egui::RichText::new(format!("({shortcut})")).small().weak());
                    }
                    response
                })
                .inner
                .clicked();
            if clicked {
                state.active_view = view;
            }
        }
    });
}

/// 渲染内容区域
#[allow(clippy::too_many_arguments)]
fn render_content(
    ui: &mut egui::Ui,
    state: &mut DebugPanelState,
    stepping: &mut bevy::ecs::schedule::Stepping,
    stepping_state: &mut DebugSteppingState,
    debug_overlay: &mut DebugOverlay,
    battle_record: &crate::core::battle::BattleRecord,
    turn_order: &crate::core::turn::TurnOrder,
    combat_intent: &crate::core::battle::CombatIntent,
    terrain_grid: &crate::core::map::runtime::TerrainGrid,
    terrain_registry: &crate::core::map::TerrainRegistry,
    occupancy: &crate::core::map::runtime::OccupancyGrid,
    grid_viewer_state: &mut viewers::GridViewerState,
    settings: &mut crate::ui::settings::GameSettings,
    tag_registry: &crate::core::tag_def::TagRegistry,
    units: &Query<(
        Entity,
        &crate::core::character::Unit,
        &crate::core::character::UnitName,
        &crate::core::character::GridPosition,
        &crate::core::attribute::Attributes,
        &crate::core::equipment::EquipmentSlots,
        &crate::core::character::TraitCollection,
        &crate::core::ability::SkillSlots,
        &crate::core::ability::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::core::character::AiBehaviorId>,
        Option<&crate::core::buff::ActiveBuffs>,
    )>,
    unit_names: &Query<&crate::core::character::UnitName>,
) {
    ui.vertical(|ui| {
        ui.set_min_width(400.0);
        egui::ScrollArea::vertical().show(ui, |ui| match state.active_view {
            DebugView::Battle => {
                viewers::battle_debugger::render(ui, battle_record, turn_order, units);
            }
            DebugView::Buff => {
                viewers::buff_viewer::render(ui, units);
            }
            DebugView::Overlay => {
                overlay::render(ui, debug_overlay);
            }
            DebugView::DamageAttribute => {
                viewers::damage_attribute_viewer::render(
                    ui,
                    &mut state.damage_attribute_tab,
                    battle_record,
                    units,
                    unit_names,
                );
            }
            DebugView::TurnQueue => {
                viewers::turn_queue_viewer::render(ui, turn_order, units);
            }
            DebugView::Stepping => {
                stepping_control::render(ui, stepping, stepping_state);
            }
            DebugView::Grid => {
                viewers::grid_viewer::render(
                    ui,
                    terrain_grid,
                    terrain_registry,
                    occupancy,
                    units,
                    grid_viewer_state,
                );
            }
            DebugView::Ai => {
                viewers::ai_viewer::render(ui, turn_order, combat_intent, &tag_registry, units);
            }
            DebugView::Equipment => {
                viewers::equipment_viewer::render(ui, units);
            }
            DebugView::Settings => {
                viewers::settings_viewer::render(ui, settings);
            }
        });
    });
}

/// 自定义 World Inspector（默认收起，F12 切换显隐）
pub fn world_inspector_ui(world: &mut World) {
    world.resource_scope(|world, mut state: Mut<WorldInspectorState>| {
        if !state.open {
            return;
        }

        let egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world);
        let Ok(egui_context) = egui_context else {
            return;
        };
        let mut egui_context = egui_context.clone();

        egui::Window::new("World Inspector")
            .default_size((320.0, 160.0))
            .open(&mut state.open)
            .default_open(false)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
                    ui.allocate_space(ui.available_size());
                });
            });
    });
}
