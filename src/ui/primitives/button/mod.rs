//! Module Name: Button Widget — 按钮原子组件
//!
//! 提供 Primary/Danger/Secondary/Ghost 四种变体的按钮。
//! 使用 Factory 模式创建，唯一入口为 spawn_button()。
//! 交互状态通过 button_interaction_system 每帧更新。
//!
//! Contract:
//!   Props (input):    label, variant, disabled（通过 ButtonState）
//!   Events (output):  ButtonClicked（Observer 模式）
//!   Local State:      ButtonInteraction（hovered, pressed, just_clicked）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §2

pub mod components;
pub mod events;
pub mod factory;
pub mod systems;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use self::components::{ButtonInteraction, ButtonState};
use self::events::ButtonClicked;
use self::systems::button_interaction_system;

/// ButtonPlugin — 注册 Button Widget 所需的 Component/Event/System
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ButtonState>()
            .register_type::<ButtonInteraction>()
            // ButtonClicked 是 Event，Bevy 0.19 自动注册（无需 add_event）
            .add_systems(Update, button_interaction_system);
    }
}
