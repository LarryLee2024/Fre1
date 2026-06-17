// Debug Stepping 控制面板：逐步执行 System，调试 Buff 链/Observer 链/回合流程
// 遵循铁律：关键系统必须支持单步执行与状态回溯
//
// 快捷键：
//   F6 : 暂停/继续（切换 Stepping 启用状态）
//   F7 : 单步执行（执行下一个 System）

use bevy::ecs::schedule::Stepping;
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// Debug Stepping 状态追踪 Resource
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DebugSteppingState {
    /// 是否曾经启用过 Stepping
    pub was_enabled: bool,
    /// 单步执行次数
    pub step_count: u32,
    /// 启用/禁用次数
    pub toggle_count: u32,
}

/// 渲染 Stepping 视图内容
pub fn render(
    ui: &mut bevy_inspector_egui::egui::Ui,
    stepping: &mut Stepping,
    stepping_state: &mut DebugSteppingState,
) {
    ui.heading("Debug Stepping");
    ui.label("System 单步调试 (F6 暂停/继续, F7 单步)");
    ui.separator();

    if stepping.is_enabled() {
        ui.colored_label(egui::Color32::YELLOW, "⏸ 已暂停");

        ui.horizontal(|ui| {
            if ui.button("▶ 继续").clicked() {
                stepping.continue_frame();
            }
            if ui.button("⏭ 单步").clicked() {
                stepping.step_frame();
                stepping_state.step_count += 1;
            }
            if ui.button("⏹ 停用").clicked() {
                stepping.disable();
                stepping_state.toggle_count += 1;
            }
        });

        if let Some((schedule, node_id)) = stepping.cursor() {
            ui.separator();
            ui.label(format!("当前: {schedule:?}[{node_id:?}]"));
        }
    } else {
        ui.colored_label(egui::Color32::GREEN, "▶ 运行中");
        if ui.button("⏸ 启用 Stepping").clicked() {
            stepping_state.was_enabled = true;
            stepping_state.toggle_count += 1;
            stepping
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .add_schedule(PostUpdate)
                .enable();
        }
    }

    ui.separator();
    ui.label(format!(
        "统计: 切换{}次, 单步{}步",
        stepping_state.toggle_count, stepping_state.step_count
    ));
}
