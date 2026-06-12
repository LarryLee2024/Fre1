// 调试工具模块：统一调试面板 + Gizmos 可视化
// 使用 bevy_egui 实现运行时可视化调试，Gizmos 实现游戏内覆盖层
//
// ── 调试快捷键 ──
// F1  : 切换统一调试面板显隐
// F2  : 切换到 Buff 视图
// F3  : Debug Overlay 全部切换（Gizmos 可视化）
// F4  : 切换到 Damage & Attribute 视图
// F5  : 切换到 Turn Queue 视图
// F6  : Debug Stepping 暂停/继续
// F7  : Debug Stepping 单步执行
// F12 : World Inspector（bevy-inspector-egui）
//
// ── 面板布局 ──
// 统一调试面板采用侧边栏导航 + 内容区域布局：
// ┌─────────────────────────────────────────────┐
// │  Debug Panel (F1 切换)                       │
// ├──────────┬──────────────────────────────────┤
// │  导航栏   │          内容区域                 │
// │  ▸ Battle │  [当前选中视图的渲染内容]          │
// │  ▸ Buff   │                                  │
// │  ▸ Overlay│                                  │
// │  ...     │                                  │
// └──────────┴──────────────────────────────────┘
//
// ── bevy_remote ──
// RemotePlugin 已注册，提供 BRP 协议核心能力
//
// ── track_location ──
// 编译时 feature，自动在 System 错误信息中标注来源文件和行号

mod gizmos_viz;
pub mod overlay;
mod stepping_control;
mod viewers;

use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// 调试视图枚举
#[derive(Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Debug)]
pub enum DebugView {
    #[default]
    Battle,
    Buff,
    Overlay,
    DamageAttribute,
    TurnQueue,
    Stepping,
    Grid,
    Ai,
    Equipment,
    Settings,
}

impl DebugView {
    /// 所有视图及其标签和快捷键
    pub fn all() -> &'static [(DebugView, &'static str, &'static str)] {
        &[
            (Self::Battle, "Battle", "F1"),
            (Self::Buff, "Buff", "F2"),
            (Self::Overlay, "Overlay", "F3"),
            (Self::DamageAttribute, "Damage", "F4"),
            (Self::TurnQueue, "Turn", "F5"),
            (Self::Stepping, "Stepping", "F6"),
            (Self::Grid, "Grid", ""),
            (Self::Ai, "AI", ""),
            (Self::Equipment, "Equip", ""),
            (Self::Settings, "Settings", ""),
        ]
    }
}

/// 统一调试面板状态
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DebugPanelState {
    /// 主面板显隐（F1 控制）
    pub show_panel: bool,
    /// 当前选中的导航项
    pub active_view: DebugView,
    /// Damage & Attribute 面板内的 Tab 切换（0=Damage, 1=Attribute）
    pub damage_attribute_tab: u32,
}

impl Default for DebugPanelState {
    fn default() -> Self {
        Self {
            show_panel: true,
            active_view: DebugView::Battle,
            damage_attribute_tab: 0,
        }
    }
}

/// 快捷键处理系统
pub fn debug_hotkey_system(
    mut state: ResMut<DebugPanelState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // F1: 切换主面板显隐
    if keyboard.just_pressed(KeyCode::F1) {
        state.show_panel = !state.show_panel;
        return;
    }

    // F6/F7: Stepping 控制（无论面板是否打开都生效）
    if keyboard.just_pressed(KeyCode::F6) {
        state.active_view = DebugView::Stepping;
        if !state.show_panel {
            state.show_panel = true;
        }
    }
    if keyboard.just_pressed(KeyCode::F7) {
        state.active_view = DebugView::Stepping;
        if !state.show_panel {
            state.show_panel = true;
        }
    }

    // 仅在面板打开时处理视图切换快捷键
    if !state.show_panel {
        return;
    }

    // F2-F5: 切换视图
    if keyboard.just_pressed(KeyCode::F2) {
        state.active_view = DebugView::Buff;
    }
    if keyboard.just_pressed(KeyCode::F3) {
        state.active_view = DebugView::Overlay;
    }
    if keyboard.just_pressed(KeyCode::F4) {
        state.active_view = DebugView::DamageAttribute;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        state.active_view = DebugView::TurnQueue;
    }
}

/// 调试工具插件
///
/// 包含：统一调试面板、Gizmos 可视化、Debug Stepping、bevy_remote
/// 所有调试功能仅在开发模式下启用
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(viewers::GridViewerState::default())
            .insert_resource(overlay::DebugOverlay::default())
            .insert_resource(DebugPanelState::default())
            .insert_resource(stepping_control::DebugSteppingState::default())
            .register_type::<overlay::DebugOverlay>()
            .register_type::<DebugPanelState>()
            .register_type::<stepping_control::DebugSteppingState>()
            // 快捷键处理
            .add_systems(PreUpdate, debug_hotkey_system)
            // 调试按钮（始终显示在右下角）
            .add_systems(PostUpdate, debug_toggle_button)
            // 统一调试面板：PostUpdate 中运行
            .add_systems(PostUpdate, unified_debug_panel)
            // Gizmos 可视化：Last 中运行，确保在所有逻辑更新之后绘制
            .add_systems(
                Last,
                (
                    gizmos_viz::debug_pathfinding,
                    gizmos_viz::debug_ai_intent,
                    gizmos_viz::debug_occupancy,
                    gizmos_viz::debug_range_outline,
                ),
            );

        // Debug Stepping
        app.init_resource::<bevy::ecs::schedule::Stepping>()
            .add_systems(bevy::app::Main, bevy::ecs::schedule::Stepping::begin_frame);

        // bevy_remote
        app.add_plugins(RemotePlugin::default());
    }
}

/// 调试按钮系统：在屏幕右下角显示一个始终可见的按钮，点击打开调试面板
fn debug_toggle_button(
    mut state: ResMut<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    // 在屏幕右下角显示一个小按钮
    egui::Area::new(egui::Id::new("debug_toggle_area"))
        .fixed_pos(egui::pos2(
            ctx.screen_rect().width() - 80.0,
            ctx.screen_rect().height() - 40.0,
        ))
        .show(ctx, |ui| {
            let label = if state.show_panel {
                "Hide Debug"
            } else {
                "Show Debug"
            };
            if ui.button(label).clicked() {
                state.show_panel = !state.show_panel;
            }
        });
}

/// 统一调试面板系统
#[allow(clippy::too_many_arguments)]
fn unified_debug_panel(
    mut state: ResMut<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    mut stepping: ResMut<bevy::ecs::schedule::Stepping>,
    mut stepping_state: ResMut<stepping_control::DebugSteppingState>,
    mut overlay: ResMut<overlay::DebugOverlay>,
    keyboard: Res<ButtonInput<KeyCode>>,
    battle_record: Res<crate::battle::BattleRecord>,
    turn_order: Res<crate::turn::TurnOrder>,
    combat_intent: Res<crate::battle::CombatIntent>,
    terrain_grid: Res<crate::map::runtime::TerrainGrid>,
    terrain_registry: Res<crate::map::TerrainRegistry>,
    occupancy: Res<crate::map::runtime::OccupancyGrid>,
    mut grid_viewer_state: ResMut<viewers::GridViewerState>,
    mut settings: ResMut<crate::ui::settings::GameSettings>,
    units: Query<(
        Entity,
        &crate::character::Unit,
        &crate::character::UnitName,
        &crate::character::GridPosition,
        &crate::core::attribute::Attributes,
        &crate::equipment::EquipmentSlots,
        &crate::character::TraitCollection,
        &crate::skill::SkillSlots,
        &crate::skill::SkillCooldowns,
        &crate::core::tag::GameplayTags,
        Option<&crate::character::AiBehaviorId>,
        Option<&crate::buff::ActiveBuffs>,
    )>,
    unit_names: Query<&crate::character::UnitName>,
) {
    // F3: Overlay 全部切换
    if keyboard.just_pressed(KeyCode::F3) && state.show_panel {
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;
    }

    // F6: Stepping 暂停/继续
    if keyboard.just_pressed(KeyCode::F6) {
        stepping_state.toggle_count += 1;
        if stepping.is_enabled() {
            stepping.disable();
        } else {
            stepping_state.was_enabled = true;
            stepping
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .add_schedule(PostUpdate)
                .enable();
        }
    }

    // F7: Stepping 单步
    if keyboard.just_pressed(KeyCode::F7) && stepping.is_enabled() {
        stepping.step_frame();
        stepping_state.step_count += 1;
    }

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
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(80.0);
                    for &(view, label, shortcut) in DebugView::all() {
                        let is_selected = state.active_view == view;
                        let response = ui.selectable_label(is_selected, label);
                        if response.clicked() {
                            state.active_view = view;
                        }
                        if !shortcut.is_empty() {
                            ui.small(format!("({})", shortcut));
                        }
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_min_width(400.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match state.active_view {
                            DebugView::Battle => {
                                viewers::battle_debugger::render(ui, &battle_record, &turn_order, &units);
                            }
                            DebugView::Buff => {
                                viewers::buff_viewer::render(ui, &units);
                            }
                            DebugView::Overlay => {
                                overlay::render(ui, &mut overlay);
                            }
                            DebugView::DamageAttribute => {
                                viewers::damage_attribute_viewer::render(
                                    ui,
                                    &mut state.damage_attribute_tab,
                                    &battle_record,
                                    &units,
                                    &unit_names,
                                );
                            }
                            DebugView::TurnQueue => {
                                viewers::turn_queue_viewer::render(ui, &turn_order, &units);
                            }
                            DebugView::Stepping => {
                                stepping_control::render(ui, &mut stepping, &mut stepping_state);
                            }
                            DebugView::Grid => {
                                viewers::grid_viewer::render(
                                    ui,
                                    &terrain_grid,
                                    &terrain_registry,
                                    &occupancy,
                                    &units,
                                    &mut grid_viewer_state,
                                );
                            }
                            DebugView::Ai => {
                                viewers::ai_viewer::render(ui, &turn_order, &combat_intent, &units);
                            }
                            DebugView::Equipment => {
                                viewers::equipment_viewer::render(ui, &units);
                            }
                            DebugView::Settings => {
                                viewers::settings_viewer::render(ui, &mut settings);
                            }
                        }
                    });
                });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_panel_state_default_all_off() {
        let state = DebugPanelState::default();
        assert!(state.show_panel);
        assert_eq!(state.active_view, DebugView::Battle);
        assert_eq!(state.damage_attribute_tab, 0);
    }

    #[test]
    fn debug_view_all_returns_correct_count() {
        assert_eq!(DebugView::all().len(), 10);
    }

    #[test]
    fn debug_view_default_is_battle() {
        assert_eq!(DebugView::default(), DebugView::Battle);
    }

    #[test]
    fn debug_view_equality() {
        assert_eq!(DebugView::Battle, DebugView::Battle);
        assert_ne!(DebugView::Battle, DebugView::Buff);
    }

    #[test]
    fn debug_view_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for &(view, _, _) in DebugView::all() {
            set.insert(view);
        }
        assert_eq!(set.len(), 10);
    }
}
