//! SelectionPlugin — 选择状态管理插件
//!
//! 注册 SelectionState（五态）和 PickContext 资源，
//! 以及 PickIntent → Domain Event 桥接 Observer。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

use super::bridge::on_pick_intent;
use super::pick_context::PickContext;
use super::state::SelectionState;

/// Selection 状态管理插件
///
/// - 注册 SelectionState Resource（五态状态机）
/// - 注册 PickContext Resource（交互模式）
/// - 注册 PickIntent → Domain Event 桥接 Observer
///
/// 必须在 PickingUiPlugin 之后注册。
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .init_resource::<PickContext>()
            .add_observer(on_pick_intent);
    }
}
