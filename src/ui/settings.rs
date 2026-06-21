//! UiSettings — 跨会话持久化 UI 设置

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::theme::switch::ThemeVariant;

/// UI 设置（Level 1 — 跨会话持久化）
#[derive(Resource, Serialize, Deserialize, Debug, Clone, Reflect)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct UiSettings {
    /// 主题
    pub theme: ThemeVariant,
    /// 语言
    pub language: String,
    /// 显示伤害数字
    pub show_damage_numbers: bool,
    /// 战斗速度倍率
    pub battle_speed: f32,
    /// 工具提示延迟（秒）
    pub tooltip_delay: f32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::Dark,
            language: "en-US".into(),
            show_damage_numbers: true,
            battle_speed: 1.0,
            tooltip_delay: 0.3,
        }
    }
}

/// 从磁盘加载设置
pub fn load_settings() -> UiSettings {
    std::fs::read_to_string("ui_settings.ron")
        .ok()
        .and_then(|data| ron::from_str(&data).ok())
        .unwrap_or_default()
}

/// 保存设置到磁盘
pub fn save_settings(settings: &UiSettings) {
    if let Ok(data) = ron::to_string(settings) {
        let _ = std::fs::write("ui_settings.ron", data);
    }
}
