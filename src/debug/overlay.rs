// 调试覆盖层资源：控制 Gizmos 可视化开关
// 遵循铁律：Inspector、Replay、Debug Panel 优先于日志堆砌

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// 调试覆盖层开关：控制各类 Gizmos 调试可视化的显示
#[derive(Resource, Default, Reflect)]
pub struct DebugOverlay {
    /// 显示寻路路径（当前移动单位的路径）
    pub show_pathfinding: bool,
    /// 显示 AI 决策（移动目标、攻击目标）
    pub show_ai_intent: bool,
    /// 显示占用网格（被占据的格子标记）
    pub show_occupancy: bool,
    /// 显示攻击/移动范围轮廓线（补充 Sprite 填充高亮）
    pub show_range_outline: bool,
}

/// 调试覆盖层面板：egui 窗口控制各 Gizmos 可视化开关
pub fn debug_overlay_panel(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    mut overlay: ResMut<DebugOverlay>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // F3 快捷键：切换全部调试覆盖层
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

    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Debug Overlay")
        .default_pos([740.0, 10.0])
        .default_size([200.0, 160.0])
        .show(ctx, |ui| {
            ui.label("Gizmos 可视化开关 (F3 全部切换)");
            ui.separator();
            ui.checkbox(&mut overlay.show_pathfinding, "寻路路径");
            ui.checkbox(&mut overlay.show_ai_intent, "AI 决策");
            ui.checkbox(&mut overlay.show_occupancy, "占用网格");
            ui.checkbox(&mut overlay.show_range_outline, "范围轮廓");
        });
}
