//! Module Name: Text Widget — 文本显示原子组件
//!
//! 提供 Body/Heading/Title/Caption/Label/Mono 六种变体的文本显示组件。
//! 使用 Factory 模式创建，唯一入口为 spawn_text()。
//! 内容同步通过 text_update_system 执行。
//!
//! Contract:
//!   Props (input):    variant, content, color_override（通过 TextWidget）
//!   Events (output):  无（纯展示组件）
//!   Local State:      无
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §4

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::TextVariant;
use self::components::TextWidget;
use self::systems::text_update_system;

/// TextPlugin — 注册 Text Widget 所需的 Component/System
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TextWidget>()
            .register_type::<TextVariant>()
            .add_systems(Update, text_update_system);
    }
}
