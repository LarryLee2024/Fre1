//! Module Name: TextInput Widget — 文本输入原子组件
//!
//! 提供一个单行文本输入组件，包含焦点状态管理和键盘输入处理。
//! 使用 Factory 模式创建，唯一入口为 spawn_text_input()。
//! 键盘输入通过 text_input_system 处理字符和退格操作。
//!
//! Contract:
//!   Props (input):    placeholder_key, max_length, value（通过 TextInputState）
//!   Events (output):  TextInputState.value 变化（外部通过 Changed<TextInputState> 检测）
//!   Local State:      无
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §TextInput

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::TextInputState;
use self::systems::text_input_system;

/// TextInputPlugin — 注册 TextInput Widget 所需的 Component/System
pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TextInputState>()
            .add_systems(Update, text_input_system);
    }
}
