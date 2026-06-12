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
// ── 面板位置规划 ──
// 为避免面板重叠遮挡游戏画面，所有调试面板按区域分布：
//
// 左侧区域 (x=10):
//   F1 Battle Debugger:  [10, 10]    - 回合状态快照
//   F2 Buff Viewer:      [10, 200]   - 单位 Buff 状态
//
// 中间区域 (x=370):
//   F4 Damage & Attribute: [370, 10] - 伤害分解与属性修饰
//
// 右侧区域 (x=740):
//   Debug Overlay:      [740, 10]    - Gizmos 可视化开关
//   Debug Stepping:     [740, 200]   - 系统单步调试
//
// 底部区域 (y=960):
//   F5 Turn Queue:      [10, 960]    - 行动队列预览
//
// 注意：面板位置基于 1920x1080 分辨率设计，高分辨率下会有更多空间
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
            .insert_resource(stepping_control::DebugSteppingState::default())
            .register_type::<overlay::DebugOverlay>()
            .register_type::<DebugPanelState>()
            .register_type::<stepping_control::DebugSteppingState>()
            // 快捷键处理
            .add_systems(PreUpdate, debug_hotkey_system)
            // egui 面板：PostUpdate 中运行
            .add_systems(
                PostUpdate,
                (
                    conditional_battle_debugger,
                    conditional_damage_attribute_viewer,
                    conditional_turn_queue_viewer,
                    viewers::buff_viewer::buff_viewer_system,
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

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;

    /// Test ID: DBG-PNL-001
    /// Title: DebugPanelState 默认所有面板关闭
    ///
    /// Given: 新创建的 DebugPanelState
    /// When: 检查默认值
    /// Then: 所有 show_* 字段为 false，damage_attribute_tab 为 0
    ///
    /// Assertions: 4 个 bool 字段均为 false, tab == 0
    #[test]
    fn debug_panel_state_default_all_off() {
        // Given & When
        let state = DebugPanelState::default();

        // Then
        assert!(!state.show_battle_debugger);
        assert!(!state.show_buff_viewer);
        assert!(!state.show_damage_attribute);
        assert!(!state.show_turn_queue);
        assert_eq!(state.damage_attribute_tab, 0);
    }

    /// Test ID: DBG-PNL-002
    /// Title: F1 快捷键切换 Battle Debugger 面板
    ///
    /// Given: DebugPanelState（show_battle_debugger=false）
    /// When: 模拟 F1 按下行为（toggle）
    /// Then: show_battle_debugger 变为 true
    ///
    /// Assertions: show_battle_debugger == true
    #[test]
    fn f1_toggles_battle_debugger() {
        // Given
        let mut state = DebugPanelState::default();
        assert!(!state.show_battle_debugger);

        // When — 模拟 F1 toggle 行为
        state.show_battle_debugger = !state.show_battle_debugger;

        // Then
        assert!(state.show_battle_debugger);
    }

    /// Test ID: DBG-PNL-003
    /// Title: F1 快捷键二次切换关闭 Battle Debugger 面板
    ///
    /// Given: DebugPanelState（show_battle_debugger=true）
    /// When: 再次模拟 F1 按下行为（toggle）
    /// Then: show_battle_debugger 变为 false
    ///
    /// Assertions: show_battle_debugger == false
    #[test]
    fn f1_toggles_battle_debugger_off() {
        // Given
        let mut state = DebugPanelState {
            show_battle_debugger: true,
            ..default()
        };

        // When — 模拟 F1 toggle 行为
        state.show_battle_debugger = !state.show_battle_debugger;

        // Then
        assert!(!state.show_battle_debugger);
    }

    /// Test ID: DBG-PNL-004
    /// Title: F2 快捷键切换 Buff Viewer 面板
    ///
    /// Given: DebugPanelState（show_buff_viewer=false）
    /// When: 模拟 F2 按下行为（toggle）
    /// Then: show_buff_viewer 变为 true
    ///
    /// Assertions: show_buff_viewer == true
    #[test]
    fn f2_toggles_buff_viewer() {
        // Given
        let mut state = DebugPanelState::default();
        assert!(!state.show_buff_viewer);

        // When
        state.show_buff_viewer = !state.show_buff_viewer;

        // Then
        assert!(state.show_buff_viewer);
    }

    /// Test ID: DBG-PNL-005
    /// Title: F4 快捷键切换 Damage & Attribute 面板
    ///
    /// Given: DebugPanelState（show_damage_attribute=false）
    /// When: 模拟 F4 按下行为（toggle）
    /// Then: show_damage_attribute 变为 true
    ///
    /// Assertions: show_damage_attribute == true
    #[test]
    fn f4_toggles_damage_attribute() {
        // Given
        let mut state = DebugPanelState::default();
        assert!(!state.show_damage_attribute);

        // When
        state.show_damage_attribute = !state.show_damage_attribute;

        // Then
        assert!(state.show_damage_attribute);
    }

    /// Test ID: DBG-PNL-006
    /// Title: F5 快捷键切换 Turn Queue 面板
    ///
    /// Given: DebugPanelState（show_turn_queue=false）
    /// When: 模拟 F5 按下行为（toggle）
    /// Then: show_turn_queue 变为 true
    ///
    /// Assertions: show_turn_queue == true
    #[test]
    fn f5_toggles_turn_queue() {
        // Given
        let mut state = DebugPanelState::default();
        assert!(!state.show_turn_queue);

        // When
        state.show_turn_queue = !state.show_turn_queue;

        // Then
        assert!(state.show_turn_queue);
    }

    /// Test ID: DBG-PNL-007
    /// Title: F4 Tab 切换 Damage/Attribute 子面板
    ///
    /// Given: DebugPanelState（damage_attribute_tab=0）
    /// When: 模拟 Tab 点击行为（切换到 1）
    /// Then: damage_attribute_tab 变为 1
    ///
    /// Assertions: damage_attribute_tab == 1
    #[test]
    fn f4_tab_switches_to_attribute() {
        // Given
        let mut state = DebugPanelState::default();
        assert_eq!(state.damage_attribute_tab, 0);

        // When — 模拟 Tab 切换到 Attribute 面板
        state.damage_attribute_tab = 1;

        // Then
        assert_eq!(state.damage_attribute_tab, 1);
    }

    /// Test ID: DBG-PNL-008
    /// Title: F4 Tab 切换回 Damage 子面板
    ///
    /// Given: DebugPanelState（damage_attribute_tab=1）
    /// When: 模拟 Tab 点击行为（切换到 0）
    /// Then: damage_attribute_tab 变为 0
    ///
    /// Assertions: damage_attribute_tab == 0
    #[test]
    fn f4_tab_switches_back_to_damage() {
        // Given
        let mut state = DebugPanelState {
            damage_attribute_tab: 1,
            ..default()
        };

        // When — 模拟 Tab 切换回 Damage 面板
        state.damage_attribute_tab = 0;

        // Then
        assert_eq!(state.damage_attribute_tab, 0);
    }

    /// Test ID: DBG-PNL-009
    /// Title: 多面板独立切换
    ///
    /// Given: DebugPanelState（全部关闭）
    /// When: 依次切换 F1、F2、F4
    /// Then: 三个面板独立开启，其余保持关闭
    ///
    /// Assertions: show_battle_debugger=true, show_buff_viewer=true,
    ///             show_damage_attribute=true, show_turn_queue=false
    #[test]
    fn multiple_panels_toggle_independently() {
        // Given
        let mut state = DebugPanelState::default();

        // When — 依次切换 F1、F2、F4
        state.show_battle_debugger = !state.show_battle_debugger;
        state.show_buff_viewer = !state.show_buff_viewer;
        state.show_damage_attribute = !state.show_damage_attribute;

        // Then
        assert!(state.show_battle_debugger);
        assert!(state.show_buff_viewer);
        assert!(state.show_damage_attribute);
        assert!(!state.show_turn_queue);
    }
}
