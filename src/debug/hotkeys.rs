// 调试快捷键处理系统
// 所有调试快捷键统一在此处理：F1-F7, F12
// 在 PreUpdate Schedule 中运行

use bevy::prelude::*;

use super::overlay::DebugOverlay;
use super::state::{DebugPanelState, DebugView, WorldInspectorState};
use super::stepping_control::DebugSteppingState;

/// 快捷键处理系统
pub fn debug_hotkey_system(
    mut state: ResMut<DebugPanelState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut stepping: ResMut<bevy::ecs::schedule::Stepping>,
    mut stepping_state: ResMut<DebugSteppingState>,
    mut world_inspector: ResMut<WorldInspectorState>,
    mut overlay: ResMut<DebugOverlay>,
) {
    // F1: 切换主面板显隐
    if keyboard.just_pressed(KeyCode::F1) {
        state.show_panel = !state.show_panel;
        return;
    }

    // F3: Overlay 全部切换（无论面板是否打开都生效）
    if keyboard.just_pressed(KeyCode::F3) {
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;
    }

    // F6: Stepping 暂停/继续 + 切换到 Stepping 视图
    if keyboard.just_pressed(KeyCode::F6) {
        state.active_view = DebugView::Stepping;
        if !state.show_panel {
            state.show_panel = true;
        }
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

    // F7: Stepping 单步执行 + 切换到 Stepping 视图
    if keyboard.just_pressed(KeyCode::F7) {
        state.active_view = DebugView::Stepping;
        if !state.show_panel {
            state.show_panel = true;
        }
        if stepping.is_enabled() {
            stepping.step_frame();
            stepping_state.step_count += 1;
        }
    }

    // F12: 切换 World Inspector（无论面板是否打开都生效）
    if keyboard.just_pressed(KeyCode::F12) {
        world_inspector.open = !world_inspector.open;
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
