// 调试工具模块：Buff Viewer、AI Viewer、Grid Viewer、Equipment Viewer、Gizmos 可视化
// 使用 bevy_egui 实现运行时可视化调试，Gizmos 实现游戏内覆盖层
//
// ── 调试快捷键 ──
// F1  : Battle Debugger（回合状态+当前行动单位+事件统计）
// F2  : Buff Viewer（切换显隐）
// F3  : Debug Overlay 全部切换（Gizmos 可视化）
// F4  : Damage & Attribute Debugger（Tab 切换两个子面板）
// F5  : Turn Queue Viewer（行动队列预览）
// F6  : Debug Stepping 暂停/继续
// F7  : Debug Stepping 单步执行
// F12 : World Inspector（bevy-inspector-egui）
//
// ── bevy_remote ──
// RemotePlugin 已注册，提供 BRP 协议核心能力（查询/修改 Entity 和 Resource）
// 注意：Bevy 0.18.1 的 bevy_remote 未启用 HTTP 传输层（bevy_internal 设 default-features=false）
// 如需 HTTP 远程访问，需在 Cargo.toml 中为 bevy_remote 单独启用 http feature
// 未来版本可通过 bevy_remote 直接连接 Bevy Editor
//
// ── track_location ──
// 编译时 feature，自动在 System 错误信息中标注来源文件和行号
// 无需代码，Cargo.toml 中已启用

mod gizmos_viz;
pub mod overlay;
mod stepping_control;
mod viewers;

use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// 调试面板显隐状态
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DebugPanelState {
    /// F1: Battle Debugger
    pub show_battle_debugger: bool,
    /// F2: Buff Viewer
    pub show_buff_viewer: bool,
    /// F4: Damage & Attribute Debugger
    pub show_damage_attribute: bool,
    /// F4 Tab: 当前选中的子面板（0=Damage, 1=Attribute）
    pub damage_attribute_tab: u32,
    /// F5: Turn Queue Viewer
    pub show_turn_queue: bool,
}

/// 快捷键处理系统
pub fn debug_hotkey_system(
    mut state: ResMut<DebugPanelState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        state.show_battle_debugger = !state.show_battle_debugger;
    }
    if keyboard.just_pressed(KeyCode::F2) {
        state.show_buff_viewer = !state.show_buff_viewer;
    }
    if keyboard.just_pressed(KeyCode::F4) {
        state.show_damage_attribute = !state.show_damage_attribute;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        state.show_turn_queue = !state.show_turn_queue;
    }
}

/// 调试工具插件
///
/// 包含：egui 调试面板、Gizmos 可视化、Debug Stepping、bevy_remote
/// 所有调试功能仅在开发模式下启用
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(viewers::GridViewerState::default())
            .insert_resource(overlay::DebugOverlay::default())
            .insert_resource(DebugPanelState::default())
            .register_type::<overlay::DebugOverlay>()
            .register_type::<DebugPanelState>()
            // 快捷键处理
            .add_systems(PreUpdate, debug_hotkey_system)
            // egui 面板：PostUpdate 中运行
            .add_systems(
                PostUpdate,
                (
                    conditional_buff_viewer,
                    conditional_battle_debugger,
                    conditional_damage_attribute_viewer,
                    conditional_turn_queue_viewer,
                    viewers::grid_viewer::grid_viewer_system,
                    viewers::ai_viewer::ai_viewer_system,
                    viewers::equipment_viewer::equipment_viewer_system,
                    overlay::debug_overlay_panel,
                    viewers::settings_viewer::settings_viewer_system,
                    stepping_control::stepping_control_panel,
                ),
            )
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

        // Debug Stepping：逐步执行 System，用于调试 Buff 链/Observer 链/回合流程
        // Stepping 是 Resource，需要手动初始化并注册 begin_frame 系统
        app.init_resource::<bevy::ecs::schedule::Stepping>()
            .add_systems(bevy::app::Main, bevy::ecs::schedule::Stepping::begin_frame);

        // bevy_remote：运行时远程控制台，可通过 BRP 协议查看/修改 Entity 和 Resource
        // 当前仅注册核心协议处理，HTTP 传输层需额外启用
        app.add_plugins(RemotePlugin::default());
    }
}

// ── 条件渲染面板 ──

/// Buff Viewer（F2 控制显隐）
fn conditional_buff_viewer(
    state: Res<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    units: Query<(
        Entity,
        &crate::character::Unit,
        &crate::character::UnitName,
        &crate::buff::ActiveBuffs,
    )>,
) {
    if !state.show_buff_viewer {
        return;
    }
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
                    crate::character::Faction::Player => "[友]",
                    crate::character::Faction::Enemy => "[敌]",
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

/// Battle Debugger（F1 控制显隐）
fn conditional_battle_debugger(
    state: Res<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    battle_record: Res<crate::battle::BattleRecord>,
    turn_order: Res<crate::turn::TurnOrder>,
    units: Query<(&crate::character::UnitName, &crate::character::Unit)>,
) {
    if !state.show_battle_debugger {
        return;
    }
    viewers::battle_debugger::battle_debugger_system_inner(
        egui_ctx,
        battle_record,
        turn_order,
        units,
        state.show_battle_debugger,
    );
}

/// Damage & Attribute Debugger（F4 控制显隐，Tab 切换子面板）
fn conditional_damage_attribute_viewer(
    mut state: ResMut<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    battle_record: Res<crate::battle::BattleRecord>,
    units: Query<(
        Entity,
        &crate::character::Unit,
        &crate::character::UnitName,
        &crate::core::attribute::Attributes,
        &crate::equipment::EquipmentSlots,
        &crate::character::TraitCollection,
    )>,
    unit_names: Query<&crate::character::UnitName>,
) {
    if !state.show_damage_attribute {
        return;
    }
    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Damage & Attribute Debugger")
        .default_pos([370.0, 10.0])
        .default_size([380.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let damage_selected = ui
                    .selectable_label(state.damage_attribute_tab == 0, "Damage Breakdown")
                    .clicked();
                let attr_selected = ui
                    .selectable_label(state.damage_attribute_tab == 1, "Attribute Modifier")
                    .clicked();
                if damage_selected {
                    state.damage_attribute_tab = 0;
                }
                if attr_selected {
                    state.damage_attribute_tab = 1;
                }
            });
            ui.separator();

            if state.damage_attribute_tab == 0 {
                viewers::damage_viewer::render_damage_panel(ui, &battle_record, &unit_names);
            } else {
                viewers::attribute_viewer::render_attribute_panel(ui, &units);
            }
        });
}

/// Turn Queue Viewer（F5 控制显隐）
fn conditional_turn_queue_viewer(
    state: Res<DebugPanelState>,
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    turn_order: Res<crate::turn::TurnOrder>,
    units: Query<(
        &crate::character::UnitName,
        &crate::character::Unit,
        &crate::core::attribute::Attributes,
    )>,
) {
    if !state.show_turn_queue {
        return;
    }
    viewers::turn_queue_viewer::turn_queue_viewer_system_inner(egui_ctx, turn_order, units);
}
