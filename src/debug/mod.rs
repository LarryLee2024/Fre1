// 调试工具模块：Buff Viewer、AI Viewer、Grid Viewer、Equipment Viewer
// 使用 bevy_egui 实现运行时可视化调试

mod ai_viewer;
mod buff_viewer;
mod equipment_viewer;
mod grid_viewer;

use bevy::prelude::*;

/// 调试工具插件
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(grid_viewer::GridViewerState::default())
            .add_systems(
                PostUpdate,
                (
                    buff_viewer::buff_viewer_system,
                    grid_viewer::grid_viewer_system,
                    ai_viewer::ai_viewer_system,
                    equipment_viewer::equipment_viewer_system,
                ),
            );
    }
}
