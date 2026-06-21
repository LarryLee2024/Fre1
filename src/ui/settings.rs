//! UiSettings — Cross-session UI settings persistence
//!
//! Provides the `UiSettings` resource and load/save functions using RON
//! serialization. Settings are persisted to `ui_settings.ron` in the
//! working directory.
//!
//! Loaded at startup and saved whenever settings change (theme switch,
//! language change, etc.).
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §3

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::theme::ThemeVariant;

/// UI settings resource — persisted across sessions.
///
/// Stored as a Bevy Resource. Modified via UiCommand handlers that
/// call `save_settings()` after applying changes.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct UiSettings {
    /// Active theme variant
    pub theme: ThemeVariant,
    /// Active locale identifier (e.g., "en-US", "zh-CN", "ja-JP")
    pub language: String,
    /// Whether to show floating damage/heal numbers
    pub show_damage_numbers: bool,
    /// Battle animation speed multiplier (0.5 = half speed, 2.0 = double)
    pub battle_speed: f32,
    /// Delay in seconds before showing tooltips
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

/// Load settings from `ui_settings.ron`.
///
/// Reads the settings file from the working directory. Returns
/// `UiSettings::default()` if the file does not exist or is malformed.
pub fn load_settings() -> UiSettings {
    let path = settings_file_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| ron::from_str(&data).ok())
        .unwrap_or_default()
}

/// Save settings to `ui_settings.ron`.
///
/// Writes the current settings to the working directory. Silently
/// ignores I/O errors (settings loss is not critical).
pub fn save_settings(settings: &UiSettings) {
    let path = settings_file_path();
    if let Ok(data) = ron::to_string(settings) {
        let _ = std::fs::write(&path, data);
    }
}

/// Returns the path to the settings file.
fn settings_file_path() -> std::path::PathBuf {
    std::path::PathBuf::from("ui_settings.ron")
}
