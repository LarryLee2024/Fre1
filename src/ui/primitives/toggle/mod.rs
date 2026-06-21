//! Module Name: Toggle Widget — 开关原子组件
//!
//! 提供一个可点击切换的开关输入组件，包含本地化标签和状态指示器。
//! 使用 Factory 模式创建，唯一入口为 spawn_toggle()。
//! 交互状态通过 toggle_interaction_system 每帧更新。
//!
//! Contract:
//!   Props (input):    label_key, default_label, checked, enabled（通过 ToggleState）
//!   Events (output):  ToggleState.checked 变化（外部通过 Changed<ToggleState> 检测）
//!   Local State:      ToggleIndicator（标记点击区域实体）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §Toggle

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{ToggleIndicator, ToggleState};
use self::systems::toggle_interaction_system;

/// TogglePlugin — 注册 Toggle Widget 所需的 Component/System
pub struct TogglePlugin;

impl Plugin for TogglePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ToggleState>()
            .register_type::<ToggleIndicator>()
            .add_systems(Update, toggle_interaction_system);
    }
}
