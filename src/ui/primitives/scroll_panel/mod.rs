//! Module Name: ScrollPanel Widget — 可滚动面板容器原子组件
//!
//! 提供一个带滚动功能的容器组件，包含裁剪容器和内部内容包裹器。
//! 使用 Factory 模式创建，唯一入口为 spawn_scroll_panel()。
//! 当前为结构骨架，scroll_offset 的 translate 应用由外部系统完成。
//!
//! Contract:
//!   Props (input):    max_height, padding（通过 ScrollPanelState）
//!   Events (output):  无（被动容器组件）
//!   Local State:      ScrollContent（标记内部内容包裹器）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §ScrollPanel

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{ScrollContent, ScrollPanelState};

/// ScrollPanelPlugin — 注册 ScrollPanel Widget 所需的 Component
pub struct ScrollPanelPlugin;

impl Plugin for ScrollPanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScrollPanelState>()
            .register_type::<ScrollContent>();
    }
}
