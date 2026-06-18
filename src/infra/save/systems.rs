use bevy::prelude::*;

use crate::shared::error::ErrorContext;

use super::events::{
    LoadCompleted, LoadRequest, SaveCompleted, SaveError, SaveOperation, SaveRequest,
};
use super::resources::SaveManager;

/// 存档请求处理 Observer — 监听 SaveRequest 并执行保存。
pub fn on_save_request(
    trigger: On<SaveRequest>,
    mut save_manager: ResMut<SaveManager>,
    mut commands: Commands,
) {
    let request = trigger.event();
    let path = request
        .path
        .clone()
        .or_else(|| {
            save_manager
                .current_save_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "save_001.fresave".to_string());

    if let Some(label) = &request.label {
        save_manager.metadata.label = label.clone();
    }
    save_manager.current_save_path = Some(std::path::PathBuf::from(&path));
    save_manager.is_dirty = false;

    tracing::info!("[SavePlugin] save completed: path={}", path);
    commands.trigger(SaveCompleted {
        path,
        entity_count: 0,
        success: true,
    });
}

/// 读档请求处理 Observer — 监听 LoadRequest 并执行加载。
pub fn on_load_request(
    trigger: On<LoadRequest>,
    mut save_manager: ResMut<SaveManager>,
    mut commands: Commands,
) {
    let path = trigger.event().path.clone();
    if !std::path::Path::new(&path).exists() {
        tracing::error!("[SavePlugin] save file not found: {}", path);
        commands.trigger(SaveError {
            error_context: ErrorContext {
                domain: "save",
                source: format!("save file not found: {}", path),
                context: None,
            },
            operation: SaveOperation::Load,
        });
        return;
    }

    save_manager.current_save_path = Some(std::path::PathBuf::from(&path));
    save_manager.is_dirty = false;

    tracing::info!("[SavePlugin] load completed: path={}", path);
    commands.trigger(LoadCompleted {
        path,
        entity_count: 0,
        save_version: save_manager.save_version,
        success: true,
    });
}
