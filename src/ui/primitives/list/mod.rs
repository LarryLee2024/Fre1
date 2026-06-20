//! Module Name: List Widget — 列表排列原子组件
//!
//! 提供 Vertical/Horizontal/Virtual 三种排列变体的列表容器。
//! 使用 Factory 模式创建，唯一入口为 spawn_list()。
//! 列表是被动容器，不发射事件，无交互系统。
//!
//! Contract:
//!   Props (input):    variant, spacing（通过 ListState）
//!   Events (output):  无（纯容器组件）
//!   Local State:      无
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §5

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{ListState, ListVariant};

/// ListPlugin — 注册 List Widget 所需的 Component
pub struct ListPlugin;

impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ListState>()
            .register_type::<ListVariant>();
    }
}
