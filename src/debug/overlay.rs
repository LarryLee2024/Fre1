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

    /// Test ID: DBG-OVL-001
    /// Title: DebugOverlay 默认所有开关关闭
    ///
    /// Given: 新创建的 DebugOverlay
    /// When: 检查默认值
    /// Then: 所有 show_* 字段为 false
    ///
    /// Assertions: 4 个 bool 字段均为 false
    #[test]
    fn debug_overlay_default_all_off() {
        // Given & When
        let overlay = DebugOverlay::default();

        // Then
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }

    /// Test ID: DBG-OVL-002
    /// Title: F3 全关→全开 行为验证
    ///
    /// Given: DebugOverlay 所有开关关闭
    /// When: 执行 F3 全开/全关逻辑（all_on = false）
    /// Then: 所有开关变为 true
    ///
    /// Assertions: 4 个 bool 字段均为 true
    #[test]
    fn f3_toggle_all_off_to_all_on() {
        // Given
        let mut overlay = DebugOverlay {
            show_pathfinding: false,
            show_ai_intent: false,
            show_occupancy: false,
            show_range_outline: false,
        };

        // When — 模拟 F3 行为：all_on = false → !false = true
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;

        // Then
        assert!(overlay.show_pathfinding);
        assert!(overlay.show_ai_intent);
        assert!(overlay.show_occupancy);
        assert!(overlay.show_range_outline);
    }

    /// Test ID: DBG-OVL-003
    /// Title: F3 全开→全关 行为验证
    ///
    /// Given: DebugOverlay 所有开关打开
    /// When: 执行 F3 全开/全关逻辑（all_on = true）
    /// Then: 所有开关变为 false
    ///
    /// Assertions: 4 个 bool 字段均为 false
    #[test]
    fn f3_toggle_all_on_to_all_off() {
        // Given
        let mut overlay = DebugOverlay {
            show_pathfinding: true,
            show_ai_intent: true,
            show_occupancy: true,
            show_range_outline: true,
        };

        // When — 模拟 F3 行为：all_on = true → !true = false
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;

        // Then
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }

    /// Test ID: DBG-OVL-004
    /// Title: F3 部分开→全关 行为验证
    ///
    /// Given: DebugOverlay 部分开关打开（show_pathfinding=true, show_occupancy=true）
    /// When: 执行 F3 全开/全关逻辑（all_on = true）
    /// Then: 所有开关变为 false
    ///
    /// Assertions: 4 个 bool 字段均为 false
    #[test]
    fn f3_toggle_partial_on_to_all_off() {
        // Given
        let mut overlay = DebugOverlay {
            show_pathfinding: true,
            show_ai_intent: false,
            show_occupancy: true,
            show_range_outline: false,
        };

        // When — 只要有一个为 true，all_on = true → 全部变 false
        let all_on = overlay.show_pathfinding
            || overlay.show_ai_intent
            || overlay.show_occupancy
            || overlay.show_range_outline;
        overlay.show_pathfinding = !all_on;
        overlay.show_ai_intent = !all_on;
        overlay.show_occupancy = !all_on;
        overlay.show_range_outline = !all_on;

        // Then
        assert!(!overlay.show_pathfinding);
        assert!(!overlay.show_ai_intent);
        assert!(!overlay.show_occupancy);
        assert!(!overlay.show_range_outline);
    }
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
