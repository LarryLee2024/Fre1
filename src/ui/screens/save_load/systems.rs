//! Module Name: SaveLoadScreen Systems
//!
//! Observer handler for SaveLoad screen button clicks.
//! Handles ToggleMode, Close, SelectSlot, Confirm, and Delete actions.

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::primitives::button::events::ButtonClicked;

use super::components::{SaveLoadAction, SaveLoadMode, SaveLoadScreen, SelectedSlot};

/// Observer：处理 SaveLoad 界面按钮点击。
///
/// 匹配所有 ButtonClicked 事件，检查触发实体是否带有 SaveLoadAction 组件。
/// 根据动作类型执行对应的 UI 操作。
///
/// # 动作处理
/// - `ToggleMode`: 切换根实体上的 SaveLoadMode（Save <-> Load）
/// - `Close`: 销毁所有 SaveLoadScreen 实体
/// - `SelectSlot(i)`: 更新 SelectedSlot 资源
/// - `Confirm`: 根据当前模式发出 SaveGame/LoadGame 命令
/// - `Delete`: MVP 阶段为空操作（UiCommand 尚无 DeleteSlot 变体）
pub fn on_save_load_button_clicked(
    on: On<ButtonClicked>,
    action_query: Query<&SaveLoadAction>,
    mut commands: Commands,
    root_query: Query<Entity, With<SaveLoadScreen>>,
    mut mode_query: Query<&mut SaveLoadMode>,
    mut selected_slot: ResMut<SelectedSlot>,
) {
    let entity = on.event().entity;
    let Ok(action) = action_query.get(entity) else {
        return;
    };

    match action {
        SaveLoadAction::Close => {
            for entity in &root_query {
                commands.entity(entity).despawn_recursive();
            }
        }
        SaveLoadAction::ToggleMode => {
            for root in &root_query {
                if let Ok(mut mode) = mode_query.get_mut(root) {
                    *mode = mode.toggle();
                }
            }
        }
        SaveLoadAction::SelectSlot(index) => {
            selected_slot.0 = Some(*index);
        }
        SaveLoadAction::Confirm => {
            let Some(slot) = selected_slot.0 else {
                return; // No slot selected, ignore click
            };
            // Determine current mode from root entity
            let mode = root_query
                .iter()
                .find_map(|root| mode_query.get(root).ok().copied());
            match mode {
                Some(SaveLoadMode::Save) => {
                    commands.trigger(UiCommand::SaveGame(slot as u32));
                }
                Some(SaveLoadMode::Load) => {
                    commands.trigger(UiCommand::LoadGame(slot as u32));
                }
                None => {
                    // No active SaveLoadScreen — should not happen, but handle gracefully
                    warn!("Confirm action ignored: no active SaveLoadScreen entity");
                }
            }
        }
        SaveLoadAction::Delete => {
            // TODO[P2][SaveLoad][2026-06-22]: UiCommand 尚无 DeleteSlot 变体。
            // MVP 阶段删除操作为空操作，仅日志记录。
            info!("Delete slot {:?} requested — MVP no-op", selected_slot.0);
        }
    }
}
