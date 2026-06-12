// Debug Stepping 控制面板：逐步执行 System，调试 Buff 链/Observer 链/回合流程
// 遵循铁律：关键系统必须支持单步执行与状态回溯
//
// 快捷键：
//   F6 : 暂停/继续（切换 Stepping 启用状态）
//   F7 : 单步执行（执行下一个 System）

use bevy::ecs::schedule::Stepping;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui;

/// Debug Stepping 状态追踪 Resource
///
/// 记录 Stepping 的使用历史，为调试回放和状态分析提供基础数据。
/// 独立于 Bevy 内置的 Stepping Resource，仅用于调试统计。
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

/// Debug Stepping 控制面板
///
/// 提供 System 单步调试能力：
/// - 暂停/继续：F6 或面板按钮
/// - 单步执行：F7 或面板按钮
/// - 继续到帧结束：面板按钮
/// - 显示当前游标位置（Schedule + NodeId）
pub fn stepping_control_panel(
    mut egui_ctx: Query<&mut EguiContext, With<bevy::window::PrimaryWindow>>,
    mut stepping: ResMut<Stepping>,
    mut stepping_state: ResMut<DebugSteppingState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // F6：暂停/继续
    if keyboard.just_pressed(KeyCode::F6) {
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

    // F7：单步执行
    if keyboard.just_pressed(KeyCode::F7) && stepping.is_enabled() {
        stepping.step_frame();
        stepping_state.step_count += 1;
    }

    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    egui::Window::new("Debug Stepping")
        .default_pos([740.0, 200.0])
        .default_size([220.0, 160.0])
        .show(ctx, |ui| {
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
                    }
                });

                // 显示当前游标位置
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

            // 显示统计信息
            ui.separator();
            ui.label(format!(
                "统计: 切换{}次, 单步{}步",
                stepping_state.toggle_count, stepping_state.step_count
            ));
        });
}
