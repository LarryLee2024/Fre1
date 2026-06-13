// egui 字体初始化：加载中文字体并配置 fallback
// 仅在首次运行时执行，使用 Local<bool> 防止重复加载

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContext, PrimaryEguiContext};
use bevy_inspector_egui::egui;
use std::sync::Arc;

/// 设置 egui 中文字体（仅在首次运行时执行）
pub fn setup_egui_font(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryEguiContext>>,
    mut initialized: Local<bool>,
) {
    if *initialized {
        return;
    }

    let Ok(mut ctx) = egui_ctx.single_mut() else {
        return;
    };
    let ctx = ctx.get_mut();

    let mut fonts = egui::FontDefinitions::default();

    // 读取字体文件（编译时绝对路径，避免运行时工作目录问题）
    let font_path = format!(
        "{}/assets/fonts/Arial Unicode.ttf",
        env!("CARGO_MANIFEST_DIR")
    );
    if let Ok(font_data) = std::fs::read(&font_path) {
        fonts.font_data.insert(
            "cn_font".to_string(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );

        // 将中文字体添加到所有字体族中作为 fallback
        for family in fonts.families.values_mut() {
            if !family.contains(&"cn_font".to_string()) {
                family.push("cn_font".to_string());
            }
        }

        ctx.set_fonts(fonts);
    }

    *initialized = true;
}
