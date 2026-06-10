// 游戏设置：用户偏好 + RON 持久化
// 0.19 迁移到 bevy_settings，当前自定义实现
// 与 UiTheme 职责分离：GameSettings = 用户偏好，UiTheme = 视觉常量

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 游戏设置（0.19 迁移到 bevy_settings）
#[derive(Resource, Reflect, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct GameSettings {
    pub ui: UiSettings,
    pub accessibility: AccessibilitySettings,
    pub gameplay: GameplaySettings,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UiSettings {
    /// 字体缩放倍率（1.0 = 默认）
    pub font_scale: f32,
    /// 色彩方案
    pub color_scheme: ColorScheme,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum ColorScheme {
    #[default]
    Normal,
    ColorBlindFriendly,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AccessibilitySettings {
    /// 色盲模式
    pub color_blind_mode: ColorBlindMode,
    /// 自动战斗速度倍率
    pub auto_battle_speed: f32,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum ColorBlindMode {
    #[default]
    None,
    /// 红色盲
    Protanopia,
    /// 绿色盲
    Deuteranopia,
    /// 蓝色盲
    Tritanopia,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameplaySettings {
    /// 动画速度倍率
    pub animation_speed: f32,
    /// 显示伤害数字
    pub show_damage_numbers: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            ui: UiSettings {
                font_scale: 1.0,
                color_scheme: ColorScheme::Normal,
            },
            accessibility: AccessibilitySettings {
                color_blind_mode: ColorBlindMode::None,
                auto_battle_speed: 1.0,
            },
            gameplay: GameplaySettings {
                animation_speed: 1.0,
                show_damage_numbers: true,
            },
        }
    }
}

impl GameSettings {
    /// 从 settings.ron 加载，失败则返回默认值
    pub fn load() -> Self {
        std::fs::read_to_string("settings.ron")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// 保存到 settings.ron
    pub fn save(&self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = std::fs::write("settings.ron", s);
        }
    }
}

/// 设置变更时自动保存
pub fn save_settings_on_change(settings: Res<GameSettings>, mut last_saved: Local<GameSettings>) {
    if settings.is_changed() && *settings != *last_saved {
        settings.save();
        *last_saved = settings.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_settings_default_合理默认值() {
        let s = GameSettings::default();
        assert!((s.ui.font_scale - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.ui.color_scheme, ColorScheme::Normal);
        assert_eq!(s.accessibility.color_blind_mode, ColorBlindMode::None);
        assert!((s.accessibility.auto_battle_speed - 1.0).abs() < f32::EPSILON);
        assert!((s.gameplay.animation_speed - 1.0).abs() < f32::EPSILON);
        assert!(s.gameplay.show_damage_numbers);
    }

    #[test]
    fn game_settings_ron_roundtrip() {
        let original = GameSettings::default();
        let ron_str = ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::default())
            .expect("序列化失败");
        let restored: GameSettings = ron::from_str(&ron_str).expect("反序列化失败");
        assert_eq!(original.ui.font_scale, restored.ui.font_scale);
        assert_eq!(
            original.gameplay.show_damage_numbers,
            restored.gameplay.show_damage_numbers
        );
    }

    #[test]
    fn color_blind_mode_枚举值完整() {
        let modes = [
            ColorBlindMode::None,
            ColorBlindMode::Protanopia,
            ColorBlindMode::Deuteranopia,
            ColorBlindMode::Tritanopia,
        ];
        for mode in &modes {
            let ron_str = ron::ser::to_string(mode).expect("序列化失败");
            let restored: ColorBlindMode = ron::from_str(&ron_str).expect("反序列化失败");
            assert_eq!(*mode, restored);
        }
    }
}
