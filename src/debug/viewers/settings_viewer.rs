// GameSettings 调试面板：运行时查看/修改游戏设置
// 遵循铁律：Inspector、Debug Panel 优先于日志堆砌

use crate::ui::settings::{ColorBlindMode, ColorScheme, GameSettings};
use bevy_inspector_egui::egui;

/// 渲染 Settings 视图内容
pub fn render(ui: &mut egui::Ui, settings: &mut GameSettings) {
    ui.heading("Game Settings");

    // UI 设置
    ui.collapsing("UI 设置", |ui| {
        ui.horizontal(|ui| {
            ui.label("字体缩放:");
            ui.add(
                egui::DragValue::new(&mut settings.ui.font_scale)
                    .speed(0.05)
                    .range(0.5..=2.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("色彩方案:");
            let variants = ["Normal", "ColorBlindFriendly"];
            let current = match settings.ui.color_scheme {
                ColorScheme::Normal => 0,
                ColorScheme::ColorBlindFriendly => 1,
            };
            egui::ComboBox::from_id_salt("color_scheme")
                .selected_text(variants[current])
                .show_ui(ui, |ui| {
                    for (i, &name) in variants.iter().enumerate() {
                        ui.selectable_value(
                            &mut settings.ui.color_scheme,
                            match i {
                                0 => ColorScheme::Normal,
                                _ => ColorScheme::ColorBlindFriendly,
                            },
                            name,
                        );
                    }
                });
        });
    });

    // 无障碍设置
    ui.collapsing("无障碍", |ui| {
        ui.horizontal(|ui| {
            ui.label("色盲模式:");
            let variants = ["None", "Protanopia", "Deuteranopia", "Tritanopia"];
            let current = match settings.accessibility.color_blind_mode {
                ColorBlindMode::None => 0,
                ColorBlindMode::Protanopia => 1,
                ColorBlindMode::Deuteranopia => 2,
                ColorBlindMode::Tritanopia => 3,
            };
            egui::ComboBox::from_id_salt("color_blind_mode")
                .selected_text(variants[current])
                .show_ui(ui, |ui| {
                    for (i, &name) in variants.iter().enumerate() {
                        ui.selectable_value(
                            &mut settings.accessibility.color_blind_mode,
                            match i {
                                1 => ColorBlindMode::Protanopia,
                                2 => ColorBlindMode::Deuteranopia,
                                3 => ColorBlindMode::Tritanopia,
                                _ => ColorBlindMode::None,
                            },
                            name,
                        );
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("自动战斗速度:");
            ui.add(
                egui::DragValue::new(&mut settings.accessibility.auto_battle_speed)
                    .speed(0.1)
                    .range(0.5..=5.0),
            );
        });
    });

    // 游戏玩法设置
    ui.collapsing("游戏玩法", |ui| {
        ui.horizontal(|ui| {
            ui.label("动画速度:");
            ui.add(
                egui::DragValue::new(&mut settings.gameplay.animation_speed)
                    .speed(0.1)
                    .range(0.1..=3.0),
            );
        });
        ui.checkbox(&mut settings.gameplay.show_damage_numbers, "显示伤害数字");
    });

    ui.separator();
    ui.label("设置变更会自动保存到 settings.ron");
}
