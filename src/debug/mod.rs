// 调试工具模块：Buff Viewer、AI Viewer、Grid Viewer、Equipment Viewer、Gizmos 可视化
// 使用 bevy_egui 实现运行时可视化调试，Gizmos 实现游戏内覆盖层
//
// ── 调试快捷键 ──
// F12 : World Inspector（bevy-inspector-egui）
// F3  : Debug Overlay 全部切换（Gizmos 可视化）
// F6  : Debug Stepping 暂停/继续
// F7  : Debug Stepping 单步执行
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

mod ai_viewer;
mod buff_viewer;
mod equipment_viewer;
mod gizmos_viz;
mod grid_viewer;
pub mod overlay;
mod settings_viewer;
mod stepping_control;

use bevy::prelude::*;
use bevy::remote::RemotePlugin;

/// 调试工具插件
///
/// 包含：egui 调试面板、Gizmos 可视化、Debug Stepping、bevy_remote
/// 所有调试功能仅在开发模式下启用
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(grid_viewer::GridViewerState::default())
            .insert_resource(overlay::DebugOverlay::default())
            .register_type::<overlay::DebugOverlay>()
            // egui 面板：PostUpdate 中运行
            .add_systems(
                PostUpdate,
                (
                    buff_viewer::buff_viewer_system,
                    grid_viewer::grid_viewer_system,
                    ai_viewer::ai_viewer_system,
                    equipment_viewer::equipment_viewer_system,
                    overlay::debug_overlay_panel,
                    settings_viewer::settings_viewer_system,
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
