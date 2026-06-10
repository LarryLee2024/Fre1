// UI 焦点管理：当模态面板打开时阻止游戏输入
// 使用 BlocksGameInput 标记需要阻止游戏输入的面板
// UiFocusState 资源由系统自动更新，游戏输入系统读取此资源决定是否跳过

use bevy::prelude::*;

/// 标记组件：拥有此组件的 UI 面板会阻止游戏输入（WASD、点击、快捷键）
#[derive(Component)]
pub struct BlocksGameInput;

/// UI 焦点状态：追踪是否有面板正在阻止游戏输入
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct UiFocusState {
    /// 是否有阻止游戏输入的面板打开
    pub blocks_input: bool,
}

/// 更新 UI 焦点状态：检查是否存在 BlocksGameInput 面板
pub fn update_ui_focus_state(
    blocking_panels: Query<(), With<BlocksGameInput>>,
    mut focus_state: ResMut<UiFocusState>,
) {
    let should_block = !blocking_panels.is_empty();
    if focus_state.blocks_input != should_block {
        focus_state.blocks_input = should_block;
    }
}
