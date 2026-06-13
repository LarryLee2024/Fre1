/// 调试工具模块：统一调试面板 + World Inspector + Gizmos 可视化
/// 使用 bevy_egui 实现运行时可视化调试，Gizmos 实现游戏内覆盖层

/// egui 中文字体初始化
mod egui_setup;
/// Gizmos 可视化系统
mod gizmos_viz;
/// 快捷键处理系统（F1-F7, F12）
mod hotkeys;
/// DebugOverlay 资源 + Overlay 视图渲染
pub mod overlay;
/// 面板渲染（统一调试面板 + World Inspector）
mod panel;
/// 调试状态定义（DebugView, DebugPanelState, WorldInspectorState）
mod state;
/// Debug Stepping 控制面板
mod stepping_control;
/// 各领域调试视图查看器
mod viewers;

use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;

pub use state::{DebugPanelState, DebugView, WorldInspectorState};

/// 调试工具插件
///
/// 包含：统一调试面板、World Inspector、Gizmos 可视化、Debug Stepping、bevy_remote
/// 所有调试功能仅在开发模式下启用
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(state::DebugPanelState::default())
            .insert_resource(overlay::DebugOverlay::default())
            .insert_resource(stepping_control::DebugSteppingState::default())
            .insert_resource(state::WorldInspectorState::default())
            .insert_resource(viewers::GridViewerState::default())
            .register_type::<overlay::DebugOverlay>()
            .register_type::<state::DebugPanelState>()
            .register_type::<stepping_control::DebugSteppingState>()
            // 快捷键处理：PreUpdate 中运行
            .add_systems(PreUpdate, hotkeys::debug_hotkey_system)
            // egui 系统必须在 EguiPrimaryContextPass 中运行
            .add_systems(
                EguiPrimaryContextPass,
                (
                    egui_setup::setup_egui_font,
                    panel::unified_debug_panel,
                    panel::world_inspector_ui,
                )
                    .chain(),
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

        // Debug Stepping
        app.init_resource::<bevy::ecs::schedule::Stepping>()
            .add_systems(bevy::app::Main, bevy::ecs::schedule::Stepping::begin_frame);

        // bevy_remote
        app.add_plugins(RemotePlugin::default());
    }
}
