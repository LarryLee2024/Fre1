//! Module Name: TabPanel Widget — 标签面板容器原子组件
//!
//! 提供一个标签页切换容器组件，包含标签按钮栏和内容占位区域。
//! 使用 Factory 模式创建，唯一入口为 spawn_tab_panel()。
//! 标签按钮通过 ButtonVariant::Primary（选中）和 Ghost（未选中）区分样式。
//!
//! Contract:
//!   Props (input):    tabs, default_index（通过 TabPanelState）
//!   Events (output):  外部通过 Changed<TabPanelState.active_tab> 检测切换
//!   Local State:      TabButton（标记每个标签按钮的索引）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §TabPanel

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{TabButton, TabPanelState};

/// TabPanelPlugin — 注册 TabPanel Widget 所需的 Component
pub struct TabPanelPlugin;

impl Plugin for TabPanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TabPanelState>()
            .register_type::<TabButton>();
    }
}
