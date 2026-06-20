//! Module Name: Panel Widget — Panel 容器原子组件
//!
//! 提供 6 种变体的 Panel 容器：Basic/Card/Modal/Tooltip/List/Group。
//! 使用 Factory 模式创建，唯一入口为 spawn_panel()。
//! Panel 是被动容器，不发射事件，无交互系统。
//!
//! Contract:
//!   Props (input):    variant, padded, title（通过 PanelState）
//!   Events (output):  无（纯容器组件）
//!   Local State:      无
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §4

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::PanelState;

/// PanelPlugin — 注册 Panel Widget 所需的 Component
pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PanelState>();
        // PanelVariant is a sub-field of PanelState, automatically registered
    }
}
