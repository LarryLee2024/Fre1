// 调试覆盖层资源：控制 Gizmos 可视化开关
// 遵循铁律：Inspector、Replay、Debug Panel 优先于日志堆砌

use bevy::prelude::*;
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

/// 渲染 Overlay 视图内容
pub fn render(ui: &mut egui::Ui, overlay: &mut DebugOverlay) {
    ui.heading("Debug Overlay");
    ui.label("Gizmos 可视化开关 (F3 全部切换)");
    ui.separator();
    ui.checkbox(&mut overlay.show_pathfinding, "寻路路径");
    ui.checkbox(&mut overlay.show_ai_intent, "AI 决策");
    ui.checkbox(&mut overlay.show_occupancy, "占用网格");
    ui.checkbox(&mut overlay.show_range_outline, "范围轮廓");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_overlay_default_all_off() {
        let overlay = DebugOverlay::default();
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }

    #[test]
    fn f3_toggle_all_off_to_all_on() {
        let mut overlay = DebugOverlay::default();
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;
        assert!(overlay.show_pathfinding);
        assert!(overlay.show_ai_intent);
        assert!(overlay.show_occupancy);
        assert!(overlay.show_range_outline);
    }

    #[test]
    fn f3_toggle_all_on_to_all_off() {
        let mut overlay = DebugOverlay {
            show_pathfinding: true,
            show_ai_intent: true,
            show_occupancy: true,
            show_range_outline: true,
        };
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }

    #[test]
    fn f3_toggle_partial_on_to_all_off() {
        let mut overlay = DebugOverlay {
            show_pathfinding: true,
            show_ai_intent: false,
            show_occupancy: true,
            show_range_outline: false,
        };
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }
}
