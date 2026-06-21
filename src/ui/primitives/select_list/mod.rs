//! Module Name: SelectList Widget — 选择列表原子组件
//!
//! 提供一个单选列表输入组件，包含可选条目和选中状态高亮。
//! 使用 Factory 模式创建，唯一入口为 spawn_select_list()。
//! 列表是被动容器，交互通过 ButtonPlugin 处理。
//!
//! Contract:
//!   Props (input):    items, selected_index（通过 SelectListState）
//!   Events (output):  外部通过 Changed<SelectListState> 检测选中变化
//!   Local State:      SelectListItem（标记每个条目按钮的索引）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §SelectList

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{SelectListItem, SelectListState};

/// SelectListPlugin — 注册 SelectList Widget 所需的 Component
pub struct SelectListPlugin;

impl Plugin for SelectListPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SelectListState>()
            .register_type::<SelectListItem>();
    }
}
