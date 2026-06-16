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
    /// 从 assets/settings.ron 加载，失败则返回默认值
    pub fn load() -> Self {
        std::fs::read_to_string("assets/settings.ron")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// 保存到 assets/settings.ron
    pub fn save(&self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = std::fs::write("assets/settings.ron", s);
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

    /// Test ID: UI-SET-001
    /// Title: GameSettings 默认值合理
    ///
    /// Given: GameSettings::default()
    /// When: 检查所有字段
    /// Then: 所有字段值符合预期默认值
    ///
    /// Assertions: font_scale=1.0, color_scheme=Normal, color_blind_mode=None, auto_battle_speed=1.0, animation_speed=1.0, show_damage_numbers=true
    #[test]
    fn 设置_默认值合理() {
        // Given
        let s = GameSettings::default();

        // When - 无需操作

        // Then
        assert!((s.ui.font_scale - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.ui.color_scheme, ColorScheme::Normal);
        assert_eq!(s.accessibility.color_blind_mode, ColorBlindMode::None);
        assert!((s.accessibility.auto_battle_speed - 1.0).abs() < f32::EPSILON);
        assert!((s.gameplay.animation_speed - 1.0).abs() < f32::EPSILON);
        assert!(s.gameplay.show_damage_numbers);
    }

    /// Test ID: UI-SET-002
    /// Title: GameSettings RON 序列化往返保持一致
    ///
    /// Given: 默认 GameSettings 实例
    /// When: 序列化为 RON 字符串再反序列化
    /// Then: 所有字段值保持不变
    ///
    /// Assertions: 所有字段相等
    #[test]
    fn 设置_ron序列化往返保持一致() {
        // Given
        let original = GameSettings::default();

        // When
        let ron_str = ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::default())
            .expect("序列化失败");
        let restored: GameSettings = ron::from_str(&ron_str).expect("反序列化失败");

        // Then
        assert_eq!(original.ui.font_scale, restored.ui.font_scale);
        assert_eq!(original.ui.color_scheme, restored.ui.color_scheme);
        assert_eq!(
            original.accessibility.color_blind_mode,
            restored.accessibility.color_blind_mode
        );
        assert!(
            (original.accessibility.auto_battle_speed - restored.accessibility.auto_battle_speed)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (original.gameplay.animation_speed - restored.gameplay.animation_speed).abs()
                < f32::EPSILON
        );
        assert_eq!(
            original.gameplay.show_damage_numbers,
            restored.gameplay.show_damage_numbers
        );
    }

    /// Test ID: UI-SET-003
    /// Title: ColorBlindMode 枚举值序列化反序列化完整
    ///
    /// Given: 所有 ColorBlindMode 枚举值
    /// When: 每个值序列化为 RON 再反序列化
    /// Then: 所有值保持不变
    ///
    /// Assertions: 每个枚举值序列化往返后相等
    #[test]
    fn 色盲模式_所有枚举值序列化反序列化() {
        // Given
        let modes = [
            ColorBlindMode::None,
            ColorBlindMode::Protanopia,
            ColorBlindMode::Deuteranopia,
            ColorBlindMode::Tritanopia,
        ];

        // When & Then
        for mode in &modes {
            let ron_str = ron::ser::to_string(mode).expect("序列化失败");
            let restored: ColorBlindMode = ron::from_str(&ron_str).expect("反序列化失败");
            assert_eq!(*mode, restored);
        }
    }
}
